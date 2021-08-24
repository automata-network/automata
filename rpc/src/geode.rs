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

// #[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

const RUNTIME_ERROR: i64 = 1;

#[rpc]
/// Geode RPC methods
pub trait GeodeServer<BlockHash> {
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
}

/// The geode struct shows its status
// #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, RuntimeDebug, Default, Serialize, Deserialize)]
pub struct WrappedGeode<Hash> {
    /// Geode id.
    pub id: [u8; 32],
    /// Provider id
    pub provider: [u8; 32],
    /// Assigned order hash and starting block
    pub order: Option<(Hash, Option<BlockNumber>)>,
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
}

impl<C> GeodeApi<C> {
    /// Create new `Geode` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        GeodeApi { client }
    }
}

impl<C> GeodeServer<<Block as BlockT>::Hash> for GeodeApi<C>
where
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: GeodeRuntimeApi<Block>,
{
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
}
