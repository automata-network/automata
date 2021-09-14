use automata_primitives::{AccountId, Block, BlockId, BlockNumber, Hash};
use automata_runtime::apis::GeodeApi as GeodeRuntimeApi;
use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;
use pallet_geode::{Geode, GeodeState};
use sc_light::blockchain::BlockchainHeaderBackend as HeaderBackend;
use sp_api::ProvideRuntimeApi;
use sp_runtime::{traits::Block as BlockT, RuntimeDebug};
use sp_std::{collections::btree_map::BTreeMap, prelude::*};
use std::sync::Arc;

use jsonrpc_pubsub::{typed::Subscriber, SubscriptionId, manager::SubscriptionManager};
use log::warn;
use sp_utils::mpsc::{tracing_unbounded, TracingUnboundedReceiver, TracingUnboundedSender};
use parking_lot::Mutex;
use futures::{FutureExt, TryFutureExt, TryStreamExt, StreamExt};
use jsonrpc_core::futures::{
	sink::Sink as Sink01,
	stream::Stream as Stream01,
	future::Future as Future01,
	future::Executor as Executor01,
};
use sp_runtime::print;

// #[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

const RUNTIME_ERROR: i64 = 1;

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
     fn subscribe_geode_state(&self, _: Self::Metadata, _: Subscriber<String>);
 
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
    subscribers: SharedGeodeStateSenders,
    manager: SubscriptionManager,
}

type GeodeStateStream = TracingUnboundedReceiver<String>;
type SharedGeodeStateSenders = Arc<Mutex<Vec<GeodeStateSender>>>;
type GeodeStateSender = TracingUnboundedSender<String>;

impl<C> GeodeApi<C> {
    /// Create new `Geode` with the given reference to the client.
    pub fn new(client: Arc<C>, manager: SubscriptionManager,) -> Self {
        GeodeApi { 
            client,
            subscribers: Arc::new(Mutex::new(vec![])),
            manager,
        }
    }

    /// Subscribe to a channel through which updates are sent
	pub fn subscribe(&self) -> GeodeStateStream {
		let (sender, receiver) = tracing_unbounded("mpsc_geode_state_notification_stream");
		self.subscribers.lock().push(sender);
		receiver
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
		subscriber: Subscriber<String>,
	) {
        print("subscribe_geode_state called");
        let stream = self.subscribe()
			.map(|x| Ok::<_,()>(String::from(x)))
			.map_err(|e| warn!("Notification stream error: {:?}", e))
			.compat();

        self.manager.add(subscriber, |sink| {
			let stream = stream.map(|res| Ok(res));
			sink.sink_map_err(|e| warn!("Error sending notifications: {:?}", e))
				.send_all(stream)
				.map(|_| ())
		});
		// let stream = self
		// 	// .justification_stream
		// 	.subscribe_custom()
		// 	.map(|x| Ok(Ok::<_, jsonrpc_core::Error>(String::from(x))));

		// self.manager.add(subscriber, |sink| {
		// 	stream
		// 		.forward(sink.sink_map_err(|e| warn!("Error sending notifications: {:?}", e)))
		// 		.map(|_| ())
		// });
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
