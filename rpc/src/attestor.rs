use automata_primitives::{Block, BlockId};
use automata_runtime::apis::AttestorApi as AttestorRuntimeApi;
use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sc_light::blockchain::BlockchainHeaderBackend as HeaderBackend;
use sp_api::ProvideRuntimeApi;
use sp_core::crypto::Pair;
use sp_core::sr25519::Pair as Sr25519Pair;
use sp_core::sr25519::{Public, Signature};
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

const RUNTIME_ERROR: i64 = 1;

#[rpc]
/// Attestor RPC methods
pub trait AttestorServer<BlockHash> {
    /// return the attestor list
    #[rpc(name = "attestor_list")]
    fn attestor_list(&self) -> Result<Vec<(Vec<u8>, Vec<u8>, u32)>>;
    /// return the attestor attesting a geode
    #[rpc(name = "geode_attestors")]
    fn geode_attestors(&self, geode: [u8; 32]) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;
    /// attestor notify chain for liveness update
    #[rpc(name = "attestor_notify_chain")]
    fn attestor_notify_chain(
        &self,
        attestor_notify: Vec<u8>,
        signature_raw_bytes: Vec<u8>,
    ) -> Result<bool>;
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
    fn attestor_list(&self) -> Result<Vec<(Vec<u8>, Vec<u8>, u32)>> {
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

    /// return the attestor attesting a geode
    fn geode_attestors(&self, geode: [u8; 32]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);

        let attestors = api.geode_attestors(&at, geode.into()).map_err(|e| Error {
            code: ErrorCode::ServerError(RUNTIME_ERROR),
            message: "Runtime unable to get geode attestors.".into(),
            data: Some(format!("{:?}", e).into()),
        })?;
        Ok(attestors)
    }

    /// attestor notify chain for liveness update
    fn attestor_notify_chain(
        &self,
        attestor_notify: Vec<u8>,
        signature_raw_bytes: Vec<u8>,
    ) -> Result<bool> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);

        if attestor_notify.len() < 32 {
            return Err(Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "message size incorrect.".into(),
                data: None,
            });
        }

        let mut attestor = [0u8; 32];
        attestor.copy_from_slice(&attestor_notify[0..32]); 

        let signature_raw_bytes_64;
        if signature_raw_bytes.len() == 64 {
            let ptr = signature_raw_bytes.as_ptr() as *const [u8; 64];
            unsafe { signature_raw_bytes_64 = *ptr }
        } else {
            return Err(Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "signature size incorrect.".into(),
                data: None,
            });
        }

        // validate inputs
        let pubkey = Public::from_raw(attestor);
        let signature = Signature::from_raw(signature_raw_bytes_64.clone());
        if !Sr25519Pair::verify(&signature, &attestor_notify, &pubkey) {
            return Err(Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "signature invalid.".into(),
                data: None,
            });
        }

        //submit a unsigned extrinsics into transaction pool
        let _ = api
            .unsigned_attestor_notify_chain(&at, attestor_notify, signature_raw_bytes_64)
            .map_err(|e| Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "Failed to submit unsigned extrinsics.".into(),
                data: Some(format!("{:?}", e).into()),
            });

        Ok(true)
    }
}
