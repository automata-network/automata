use automata_primitives::{AccountId, Block, BlockId};
use automata_runtime::apis::TransferApi as TransferRuntimeApi;
use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sc_light::blockchain::BlockchainHeaderBackend as HeaderBackend;
use sp_api::ProvideRuntimeApi;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;
use sp_core::{ecdsa, H160, crypto::Ss58Codec};

const RUNTIME_ERROR: i64 = 1;

#[rpc]
pub trait TransferServer<BlockHash> {
    //transfer to substrate address
    #[rpc(name = "transfer_to_substrate_account")]
    fn transfer_to_substrate_account(&self, 
        source_address: String, 
        target_address: String, 
        value: u128, 
        signature: String) -> Result<u64>;

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
        source_address: String, 
        target_address: String, 
        value: u128, 
        signature: String) -> Result<u64> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);
        let mut source_address_bytes = [0u8; 20];
        source_address_bytes.copy_from_slice(hex::decode(source_address).unwrap().as_slice());
        let mut signature_bytes = [0u8; 65];
        signature_bytes.copy_from_slice(hex::decode(signature).unwrap().as_slice());

        api.transfer_to_substrate_account(&at,
            H160::from(&source_address_bytes),
            AccountId::from_ss58check(target_address.as_str()).unwrap(),
            // AccountId::from(target_address_bytes),
            value,
            ecdsa::Signature::from_slice(&signature_bytes))
            .map_err(|e| Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "Transfer to substrate account failed.".into(),
                data: Some(format!("{:?}", e).into()),
            }).ok();
        Ok(1)
    }

    fn silly_double(&self, val: u64) -> Result<u64> {
        Ok(2 * val)
    }
}
