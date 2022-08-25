use automata_primitives::{AccountId, Block, BlockId};
use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

#[cfg(feature = "contextfree")]
use contextfree_runtime::apis::AttestorApi as AttestorRuntimeApi;
#[cfg(feature = "finitestate")]
use finitestate_runtime::apis::AttestorApi as AttestorRuntimeApi;

const RUNTIME_ERROR: i64 = 1;

#[rpc]
/// Attestor RPC methods
pub trait AttestorServer<BlockHash> {
    /// return the registered geode list
    #[rpc(name = "attestor_list")]
    fn attestor_list(&self) -> Result<Vec<(Vec<u8>, Vec<u8>, u32)>>;
    #[rpc(name = "attestor_attested_appids")]
    fn attestor_attested_appids(&self, attestor: [u8; 32]) -> Result<Vec<[u8; 32]>>;
    #[rpc(name = "attestor_heartbeat")]
    fn attestor_heartbeat(&self, message: Vec<u8>, signature_raw_bytes: Vec<u8>) -> Result<bool>;
}

pub struct AttestorApi<C> {
    client: Arc<C>,
}

impl<C> AttestorApi<C> {
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
    fn attestor_list(&self) -> Result<Vec<(Vec<u8>, Vec<u8>, u32)>> {
        use sp_api::BlockId;
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

    fn attestor_attested_appids(&self, attestor: [u8; 32]) -> Result<Vec<[u8; 32]>> {
        use sp_api::BlockId;
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);
        let attestor_attested_geodes_list = api
            .attestor_attested_appids(&at, attestor.into())
            .map_err(|e| Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "Runtime unable to get attestor attested app list.".into(),
                data: Some(format!("{:?}", e).into()),
            })?;
        let attestor_attested_geodes_list: Vec<[u8; 32]> = attestor_attested_geodes_list
            .into_iter()
            .map(|e| e.into())
            .collect();
        Ok(attestor_attested_geodes_list)
    }

    fn attestor_heartbeat(&self, message: Vec<u8>, signature_raw_bytes: Vec<u8>) -> Result<bool> {
        use sp_api::BlockId;
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);
        let mut signature = [0_u8; 64];
        signature.copy_from_slice(&signature_raw_bytes);
        let result = api
            .unsigned_attestor_heartbeat(&at, message, signature)
            .map_err(|e| Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "Runtime unable to send heartbeat.".into(),
                data: Some(format!("{:?}", e).into()),
            })?;
        Ok(result)
    }
}
