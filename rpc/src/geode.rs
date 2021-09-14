use automata_primitives::{AccountId, Block, BlockId, BlockNumber, Hash};
use automata_runtime::apis::GeodeApi as GeodeRuntimeApi;
use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;
use pallet_geode::{Geode, GeodeState};
use sc_light::blockchain::BlockchainHeaderBackend as HeaderBackend;
use sp_api::ProvideRuntimeApi;
use sp_runtime::{traits::Block as BlockT, RuntimeDebug};
use sp_std::{collections::{btree_map::BTreeMap, btree_set::BTreeSet}, prelude::*};
use std::sync::Arc;

use jsonrpc_pubsub::{typed::Subscriber, SubscriptionId, manager::SubscriptionManager};
use log::warn;
use sp_utils::mpsc::{tracing_unbounded, TracingUnboundedReceiver, TracingUnboundedSender};
use parking_lot::Mutex;
use futures::{FutureExt, TryFutureExt, TryStreamExt, StreamExt};
use jsonrpc_core::futures::{
    stream,
	sink::Sink as Sink01,
	stream::Stream as Stream01,
	future::Future as Future01,
	future::Executor as Executor01,
};
use sp_runtime::print;

// #[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

const RUNTIME_ERROR: i64 = 1;

type GeodeId = [u8; 32];
type SubscriberId = u64;

pub struct GeodeStateNotifications {
    next_id: SubscriberId,
    listeners: BTreeMap<GeodeId, BTreeSet<SubscriberId>>,
    sinks: BTreeMap<SubscriberId, GeodeStateSender>,
}

impl GeodeStateNotifications {
    pub fn new() -> Self {
        GeodeStateNotifications {
            next_id: 0,
            listeners: BTreeMap::new(),
            sinks: BTreeMap::new(),
        }
    }

    // Start subscribtion for changes in state of a particular geode
    pub fn subscribe(&mut self, geode_id: GeodeId) -> GeodeStateStream {
        self.next_id += 1;
        let subscriber_id = self.next_id;

        match self.listeners.get_mut(&geode_id) {
            Some(subscribers) => {
                subscribers.insert(subscriber_id);
            },
            None => {
                let mut subscribers = BTreeSet::new();
                subscribers.insert(subscriber_id);
                self.listeners.insert(geode_id, subscribers);
            }
        };

        // insert sink
        let (tx, rx) = tracing_unbounded("mpsc_geode_state_notification_items");
        self.sinks.insert(subscriber_id, tx);


        // if let Some(m) = self.metrics.as_ref() {
        //     m.with_label_values(&[&"added"]).inc();
        // }

        rx
    }

    // Trigger notification to all listeners
    pub fn trigger(&mut self, id: GeodeId, new_state: GeodeState) {
        // get the subscribers interested in the geode whose state is changing
        let subscribers = match self.listeners.get_mut(&id) {
            Some(subscribers) => subscribers,
            None => return,
        };
        let mut to_remove = vec!();
        for sub_id in subscribers.iter() {
            let sink = self.sinks.get_mut(&sub_id).unwrap();
            match sink.unbounded_send(new_state.clone()) {
                Ok(_) => (),
                Err(_) => to_remove.push(sub_id.clone()),
            };
        }

        for id in to_remove {
            subscribers.remove(&id);
            self.sinks.remove(&id);
        }
    }
}

#[rpc]
/// Geode RPC methods
pub trait GeodeServer<BlockHash> {
    /// RPC Metadata
	type Metadata;

    /// return the registered geode list
    #[rpc(name = "registered_geodes")]
    fn registered_geodes(&self) -> Result<Vec<WrappedGeode<Hash>>>;
    /// return the attested geode list
    #[rpc(name = "attested_geodes")]
    fn attested_geodes(&self) -> Result<Vec<WrappedGeode<Hash>>>;
    /// Return list geode an attestor is attesting
    #[rpc(name = "attestor_attested_geodes")]
    fn attestor_attested_geodes(&self, attestor: [u8; 32]) -> Result<Vec<WrappedGeode<Hash>>>;
    /// Return the current state of a geode
    #[rpc(name = "geode_state")]
    fn geode_state(&self, geode: [u8; 32]) -> Result<Option<GeodeState>>;

    /// Geode state subscription
    #[pubsub(
        subscription = "geode_state",
        subscribe,
        name = "geode_subscribeState",
     )]
     fn subscribe_geode_state(&self, _: Self::Metadata, _: Subscriber<GeodeState>, id: GeodeId);
 
     /// Unsubscribe from geode state subscription.
     #[pubsub(
         subscription = "geode_state",
         unsubscribe,
         name = "geode_unsubscribeState"
     )]
     fn unsubscribe_geode_state(&self, _: Option<Self::Metadata>, _: SubscriptionId) -> Result<bool>;
}

/// The geode struct shows its status
// #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, RuntimeDebug, Default, Serialize, Deserialize)]
pub struct WrappedGeode<Hash> {
    /// Geode id.
    pub id: [u8; 32],
    /// Provider id
    pub provider: [u8; 32],
    /// Assigned order hash
    pub order: Option<Hash>,
    /// Geode's public ip.
    pub ip: Vec<u8>,
    /// Geode's dns.
    pub dns: Vec<u8>,
    /// Geodes' properties
    pub props: BTreeMap<Vec<u8>, Vec<u8>>,
    /// Current state of the geode and the block number of since last state change
    pub state: GeodeState,
    /// promise to be online until which block
    pub promise: BlockNumber,
}

impl From<Geode<AccountId, Hash>> for WrappedGeode<Hash> {
    fn from(geode: Geode<AccountId, Hash>) -> Self {
        WrappedGeode {
            id: geode.id.into(),
            provider: geode.provider.into(),
            order: geode.order,
            ip: geode.ip,
            dns: geode.dns,
            props: geode.props,
            state: geode.state,
            promise: geode.promise,
        }
    }
}

/// An implementation of geode specific RPC methods.
pub struct GeodeApi<C> {
    client: Arc<C>,
    notifications: Mutex<GeodeStateNotifications>,
    manager: SubscriptionManager,
}

type GeodeStateStream = TracingUnboundedReceiver<GeodeState>;
type GeodeStateSender = TracingUnboundedSender<GeodeState>;

impl<C> GeodeApi<C> {
    /// Create new `Geode` with the given reference to the client.
    pub fn new(client: Arc<C>, manager: SubscriptionManager,) -> Self {
        GeodeApi { 
            client,
            notifications: Mutex::new(GeodeStateNotifications::new()),
            manager,
        }
    }
}

impl<C> GeodeServer<<Block as BlockT>::Hash> for GeodeApi<C>
where
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: GeodeRuntimeApi<Block>,
{
    type Metadata = sc_rpc::Metadata;

    /// get registered geode list
    fn registered_geodes(&self) -> Result<Vec<WrappedGeode<Hash>>> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);

        let registered_geodes_list = api.registered_geodes(&at).map_err(|e| Error {
            code: ErrorCode::ServerError(RUNTIME_ERROR),
            message: "Runtime unable to get registered geodes list.".into(),
            data: Some(format!("{:?}", e).into()),
        })?;
        let mut res = Vec::<WrappedGeode<Hash>>::new();
        for geode in registered_geodes_list {
            res.push(geode.into())
        }
        Ok(res)
    }

    /// get registered geode list
    fn attested_geodes(&self) -> Result<Vec<WrappedGeode<Hash>>> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);

        let attested_geodes_list = api.attested_geodes(&at).map_err(|e| Error {
            code: ErrorCode::ServerError(RUNTIME_ERROR),
            message: "Runtime unable to get attested geodes list.".into(),
            data: Some(format!("{:?}", e).into()),
        })?;
        let mut res = Vec::<WrappedGeode<Hash>>::new();
        for geode in attested_geodes_list {
            res.push(geode.into())
        }
        Ok(res)
    }

    /// Return list geode an attestor is attesting
    fn attestor_attested_geodes(&self, attestor: [u8; 32]) -> Result<Vec<WrappedGeode<Hash>>> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);
        let attestor_attested_geodes_list = api
            .attestor_attested_geodes(&at, attestor.into())
            .map_err(|e| Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "Runtime unable to get attestor attested geodes list.".into(),
                data: Some(format!("{:?}", e).into()),
            })?;
        let mut res = Vec::<WrappedGeode<Hash>>::new();
        for geode in attestor_attested_geodes_list {
            res.push(geode.into())
        }
        Ok(res)
    }

    /// Return the current state of a geode
    fn geode_state(&self, geode: [u8; 32]) -> Result<Option<GeodeState>> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);
        let geode_state = api.geode_state(&at, geode.into()).map_err(|e| Error {
            code: ErrorCode::ServerError(RUNTIME_ERROR),
            message: "Runtime unable to get geode state.".into(),
            data: Some(format!("{:?}", e).into()),
        })?;

        Ok(geode_state)
    }

    fn subscribe_geode_state(
		&self,
		_metadata: Self::Metadata,
		subscriber: Subscriber<GeodeState>,
        id: GeodeId,
	) {
        print("subscribe_geode_state called");
        let initial = match self.geode_state(id.clone()) {
            Ok(state) => match state {
                Some(initial) => Ok(initial),
                None => {
                    Ok(GeodeState::Registered)
                    // let _ = subscriber.reject(Error::invalid_params("no such geode"));
				    // return
                },
            }
            Err(e) => Err(e),
        };

        let stream = self.notifications.lock().subscribe(id)
			.map(|x| Ok::<_,()>(GeodeState::from(x)))
			.map_err(|e| warn!("Notification stream error: {:?}", e))
			.compat();

        self.manager.add(subscriber, |sink| {
			let stream = stream.map(|res| Ok(res));
			sink.sink_map_err(|e| warn!("Error sending notifications: {:?}", e))
				.send_all(stream::iter_result(vec![Ok(initial)])
                    .chain(stream))
				.map(|_| ())
		});
	}

	fn unsubscribe_geode_state(
		&self,
		_metadata: Option<Self::Metadata>,
		id: SubscriptionId,
	) -> Result<bool> {
        print("unsubscribe_geode_state called");
		Ok(self.manager.cancel(id))
	}
}
