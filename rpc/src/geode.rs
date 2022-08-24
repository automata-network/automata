use automata_primitives::Block;
#[cfg(feature = "contextfree")]
use contextfree_runtime::apis::GeodeApi as GeodeRuntimeApi;
#[cfg(feature = "finitestate")]
use finitestate_runtime::apis::GeodeApi as GeodeRuntimeApi;
use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::BlockId;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

const RUNTIME_ERROR: i64 = 1;

#[rpc]
pub trait GeodeServer<BlockHash> {
    #[rpc(name = "geode_ready")]
    fn geode_ready(&self, message: Vec<u8>, signature_raw_bytes: Vec<u8>) -> Result<bool>;
    #[rpc(name = "geode_finalizing")]
    fn geode_finalizing(&self, message: Vec<u8>, signature_raw_bytes: Vec<u8>) -> Result<bool>;
    #[rpc(name = "geode_finalized")]
    fn geode_finalized(&self, message: Vec<u8>, signature_raw_bytes: Vec<u8>) -> Result<bool>;
    #[rpc(name = "geode_finalize_failed")]
    fn geode_finalize_failed(&self, message: Vec<u8>, signature_raw_bytes: Vec<u8>)
        -> Result<bool>;
}

pub struct GeodeApi<C> {
    client: Arc<C>,
}

impl<C> GeodeApi<C> {
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
    fn geode_ready(&self, message: Vec<u8>, signature_raw_bytes: Vec<u8>) -> Result<bool> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);
        let mut signature = [0_u8; 64];
        signature.copy_from_slice(&signature_raw_bytes);
        let result = api
            .unsigned_geode_ready(&at, message, signature)
            .map_err(|e| Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "Runtime unable to call geode_ready.".into(),
                data: Some(format!("{:?}", e).into()),
            })?;
        Ok(result)
    }

    fn geode_finalizing(&self, message: Vec<u8>, signature_raw_bytes: Vec<u8>) -> Result<bool> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);
        let mut signature = [0_u8; 64];
        signature.copy_from_slice(&signature_raw_bytes);
        let result = api
            .unsigned_geode_finalizing(&at, message, signature)
            .map_err(|e| Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "Runtime unable to call geode_finalizing.".into(),
                data: Some(format!("{:?}", e).into()),
            })?;
        Ok(result)
    }

    fn geode_finalized(&self, message: Vec<u8>, signature_raw_bytes: Vec<u8>) -> Result<bool> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);
        let mut signature = [0_u8; 64];
        signature.copy_from_slice(&signature_raw_bytes);
        let result = api
            .unsigned_geode_finalized(&at, message, signature)
            .map_err(|e| Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "Runtime unable to geode_finalized.".into(),
                data: Some(format!("{:?}", e).into()),
            })?;
        Ok(result)
    }

    fn geode_finalize_failed(
        &self,
        message: Vec<u8>,
        signature_raw_bytes: Vec<u8>,
    ) -> Result<bool> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);
        let mut signature = [0_u8; 64];
        signature.copy_from_slice(&signature_raw_bytes);
        let result = api
            .unsigned_geode_finalize_failed(&at, message, signature)
            .map_err(|e| Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "Runtime unable to call geode_finalized.".into(),
                data: Some(format!("{:?}", e).into()),
            })?;
        Ok(result)
    }
}
