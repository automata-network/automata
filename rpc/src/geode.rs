use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;
use std::sync::Arc;
use automata_runtime::apis::GeodeApi as GeodeRuntimeApi;
use sc_light::blockchain::BlockchainHeaderBackend as HeaderBackend;
use pallet_geode::Geode;
use automata_primitives::{AccountId, Block, BlockId, Hash};
use sp_api::ProvideRuntimeApi;
use sp_runtime::traits::Block as BlockT;
use std::convert::TryInto;

const RUNTIME_ERROR: i64 = 1;

#[rpc]
/// Geode RPC methods
pub trait GeodeServer<BlockHash> {
    /// return the registered geode list
    #[rpc(name = "registered_geodes")]
    fn registered_geodes(&self) -> Result<Vec<Geode<AccountId, Hash>>>;
    /// return the attested geode list
    #[rpc(name = "attested_geodes")]
    fn attested_geodes(&self) -> Result<Vec<Geode<AccountId, Hash>>>;
    /// Return list geode an attestor is attesting
    #[rpc(name = "attestor_attested_geodes")]
    fn attestor_attested_geodes(&self, attestor: String) -> Result<Vec<Geode<AccountId, Hash>>>;
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
    fn registered_geodes(&self) -> Result<Vec<Geode<AccountId, Hash>>> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);

        let registered_geodes_list = api.registered_geodes(&at).map_err(|e| Error {
            code: ErrorCode::ServerError(RUNTIME_ERROR),
            message: "Runtime unable to get registered geodes list.".into(),
            data: Some(format!("{:?}", e).into()),
        })?;
        Ok(registered_geodes_list)
    }

    /// get registered geode list
    fn attested_geodes(&self) -> Result<Vec<Geode<AccountId, Hash>>> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);

        let attested_geodes_list = api.attested_geodes(&at).map_err(|e| Error {
            code: ErrorCode::ServerError(RUNTIME_ERROR),
            message: "Runtime unable to get attested geodes list.".into(),
            data: Some(format!("{:?}", e).into()),
        })?;
        Ok(attested_geodes_list)
    }

    /// Return list geode an attestor is attesting
    fn attestor_attested_geodes(&self, attestor: String) -> Result<Vec<Geode<AccountId, Hash>>> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);
        let att: [u8; 32] = hex::decode(attestor.as_str()).map_err(|e| Error {
            code: ErrorCode::InvalidParams,
            message: "Invalid attestor.".into(),
            data: Some(format!("{:?}", e).into()),
        })?.try_into().map_err(|e| Error {
            code: ErrorCode::InvalidParams,
            message: "Invalid attestor.".into(),
            data: Some(format!("{:?}", e).into()),
        })?;
        let attestor_attested_geodes_list = api.attestor_attested_geodes(&at, att.into()).map_err(|e| Error {
            code: ErrorCode::ServerError(RUNTIME_ERROR),
            message: "Runtime unable to get attestor attested geodes list.".into(),
            data: Some(format!("{:?}", e).into()),
        })?;
        Ok(attestor_attested_geodes_list)
    }
}