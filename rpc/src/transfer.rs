use automata_primitives::{AccountId, Block, BlockId};
use automata_runtime::apis::TransferApi as TransferRuntimeApi;
use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sc_light::blockchain::BlockchainHeaderBackend as HeaderBackend;
use sp_api::ProvideRuntimeApi;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;
use sp_core::{ecdsa, H160};

const RUNTIME_ERROR: i64 = 1;

#[rpc]
pub trait TransferServer<BlockHash> {
    //transfer to substrate address
    #[rpc(name = "transfer_to_substrate_account")]
    fn transfer_to_substrate_account(&self, 
        source_address: [u8; 20], 
        target_address: [u8; 32], 
        value: u128, 
        signature: String);

    #[rpc(name = "silly_double")]
	fn silly_double(&self, val: u64) -> Result<u64>;
}

pub struct TransferApi<C> {
    client: Arc<C>,
}

impl<C> TransferApi<C> {
    pub fn new(client: Arc<C>) -> Self {
        TransferApi { client }
    }
}

impl<C> TransferServer<<Block as BlockT>::Hash> for TransferApi<C>
where
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: TransferRuntimeApi<Block>,
{
    fn transfer_to_substrate_account(&self, 
        source_address: [u8; 20], 
        target_address: [u8; 32], 
        value: u128, 
        signature: String) 
    {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);

        api.transfer_to_substrate_account(&at, 
            H160::from(&source_address),
            AccountId::from(target_address),
            value,
            ecdsa::Signature::from_slice(signature.as_bytes()))
            .map_err(|e| Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "Transfer to substrate account failed.".into(),
                data: Some(format!("{:?}", e).into()),
            }).ok();
    }

    fn silly_double(&self, val: u64) -> Result<u64> {
        Ok(2 * val)
    }
}
