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

use sp_core::{twox_128, blake2_128};
use codec::{Encode, Decode};
use sc_client_api::BlockchainEvents;
use sp_core::storage::{StorageKey};
use sc_client_api::StorageChangeSet;

use jsonrpc_pubsub::{typed::Subscriber, SubscriptionId, manager::SubscriptionManager};
use log::warn;
use futures::{TryStreamExt, StreamExt};
use jsonrpc_core::futures::{
    stream,
    sink::Sink as Sink01,
    stream::Stream as Stream01,
    future::Future as Future01,
};

use serde::{Deserialize, Serialize};

const RUNTIME_ERROR: i64 = 1;

type GeodeId = [u8; 32];

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
    manager: SubscriptionManager,
}

impl<C> GeodeApi<C> {
    /// Create new `Geode` with the given reference to the client.
    pub fn new(client: Arc<C>, manager: SubscriptionManager,) -> Self {
        GeodeApi { 
            client,
            manager,
        }
    }
}

impl<C> GeodeServer<<Block as BlockT>::Hash> for GeodeApi<C>
where
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + BlockchainEvents<Block>,
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
        // get the current state of the geode
        // if the geode does not exist, reject the subscription
        let initial = match self.geode_state(id.clone()) {
            Ok(state) => match state {
                Some(initial) => Ok(initial),
                None => {
                    let _ = subscriber.reject(Error::invalid_params("no such geode"));
                    return
                },
            }
            Err(e) => Err(e),
        };
        let key: StorageKey = StorageKey(build_storage_key(id.clone()));
        let keys = Into::<Option<Vec<_>>>::into(vec!(key));
        let stream = match self.client.storage_changes_notification_stream(
            keys.as_ref().map(|x| &**x),
            None
        ) {
            Ok(stream) => stream,
            Err(err) => {
                let _ = subscriber.reject(client_err(err).into());
                return;
            },
        };

        let stream = stream
            .map(|(_block, changes)| Ok::<_, ()>(get_geode_state(changes)))
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
        Ok(self.manager.cancel(id))
    }
}

fn build_storage_key(id: GeodeId) -> Vec<u8> {
    let geode_module = twox_128(b"GeodeModule");
    let geodes = twox_128(b"Geodes");
    let geode: AccountId = id.into();
    let geode = blake2_128_concat(&geode.encode());

    let mut param = vec!();
    param.extend(geode_module);
    param.extend(geodes);
    param.extend(geode);
    param
}

fn blake2_128_concat(d: &[u8]) -> Vec<u8> {
    let mut v = blake2_128(d).to_vec();
    v.extend_from_slice(d);
    v
}

fn get_geode_state(changes: StorageChangeSet) -> GeodeState {
    for (_, _, data) in changes.iter() {
        match data {
            Some(data) => {
                let mut value: &[u8] = &data.0.clone();
                match GeodeState::decode(&mut value) {
                    Ok(state) => {
                        return state;
                    },
                    Err(_) => warn!("unable to decode GeodeState")
                }
            },
            None => warn!("empty change set"),
        };
    }
    GeodeState::Null
}

fn client_err(_: sp_blockchain::Error) -> Error {
    Error::invalid_request()
}
