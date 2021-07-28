use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;
use std::sync::Arc;
use automata_runtime::apis::AttestorApi as AttestorRuntimeApi;
use sc_light::blockchain::BlockchainHeaderBackend as HeaderBackend;
use automata_primitives::{AccountId, Block, BlockId, Hash};
use sp_api::ProvideRuntimeApi;
use sp_runtime::traits::Block as BlockT;

const RUNTIME_ERROR: i64 = 1;

#[rpc]
/// Attestor RPC methods
pub trait AttestorApi<BlockHash> {
    /// return the attestor list
    #[rpc(name = "attestor_list")]
    fn attestor_list(&self, at: Option<BlockHash>) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;
}

/// An implementation of contract specific RPC methods.
pub struct Attestor<C> {
    client: Arc<C>,
}

impl<C> Attestor<C> {
    /// Create new `Attestor` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Attestor { client }
    }
}

impl<C> AttestorApi<<Block as BlockT>::Hash> for Attestor<C>
where
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: AttestorRuntimeApi<Block>,
{
    /// get attestor list
    fn attestor_list(&self, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
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
}