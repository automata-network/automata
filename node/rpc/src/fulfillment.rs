use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;
use std::sync::Arc;

use automata_primitives::{AccountId, Block, BlockId, Hash};
use automata_runtime::apis::FulfillmentApi as FulfillmentRuntimeApi;
use pallet_fulfillment::Geode;
use sc_light::blockchain::BlockchainHeaderBackend as HeaderBackend;
use sp_api::ProvideRuntimeApi;
use sp_runtime::traits::Block as BlockT;

pub use self::gen_client::Client as FulfillmentClient;

const RUNTIME_ERROR: i64 = 1;

#[rpc]
/// fulfillment RPC methods
pub trait FulfillmentApi<BlockHash> {
    /// return the attestor list
    #[rpc(name = "attestor_list")]
    fn attestor_list(&self) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;

    /// return the registered_geodes
    #[rpc(name = "registered_geodes")]
    fn registered_geodes(&self) -> Result<Vec<Geode<AccountId, Hash>>>;
}

/// An implementation of contract specific RPC methods.
pub struct Fulfillment<C> {
    client: Arc<C>,
}

impl<C> Fulfillment<C> {
    /// Create new `Fulfillment` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Fulfillment { client }
    }
}

impl<C> FulfillmentApi<<Block as BlockT>::Hash> for Fulfillment<C>
where
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: FulfillmentRuntimeApi<Block>,
{
    /// get attestor list
    fn attestor_list(&self) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);

        let attestor_list = api.attestor_list(&at).map_err(|e| Error {
            code: ErrorCode::ServerError(RUNTIME_ERROR),
            message: "Runtime unable to get attestor list.".into(),
            data: Some(format!("{:?}", e).into()),
        })?;
        Ok(attestor_list)
    }

    /// get registered geodes
    fn registered_geodes(&self) -> Result<Vec<Geode<AccountId, Hash>>> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);

        let registered_geodes = api.registered_geodes(&at).map_err(|e| Error {
            code: ErrorCode::ServerError(RUNTIME_ERROR),
            message: "Runtime unable to get registered_geodes.".into(),
            data: Some(format!("{:?}", e).into()),
        })?;
        Ok(registered_geodes)
    }
}
