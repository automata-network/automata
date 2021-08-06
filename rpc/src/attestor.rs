use automata_primitives::{Block, BlockId};
use automata_runtime::apis::AttestorApi as AttestorRuntimeApi;
use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sc_light::blockchain::BlockchainHeaderBackend as HeaderBackend;
use sp_api::ProvideRuntimeApi;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

const RUNTIME_ERROR: i64 = 1;

#[rpc]
/// Attestor RPC methods
pub trait AttestorServer<BlockHash> {
    /// return the attestor list
    #[rpc(name = "attestor_list")]
    fn attestor_list(&self) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;
}

/// An implementation of attestor specific RPC methods.
pub struct AttestorApi<C> {
    client: Arc<C>,
}

impl<C> AttestorApi<C> {
    /// Create new `Attestor` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        AttestorApi { client }
    }
}

impl<C> AttestorServer<<Block as BlockT>::Hash> for AttestorApi<C>
where
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: AttestorRuntimeApi<Block>,
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
}
