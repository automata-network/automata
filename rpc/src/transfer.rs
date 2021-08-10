use automata_primitives::{AccountId, Block, BlockId};
use std::sync::Arc;
use pallet_transfer::TransferParam;
use automata_runtime::apis::TransferApi as TransferRuntimeApi;
use jsonrpc_core::{Error, ErrorCode};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_runtime::{traits::Block as BlockT};
use sc_light::blockchain::BlockchainHeaderBackend as HeaderBackend;

const RUNTIME_ERROR: i64 = 1;

#[rpc]
pub trait TransferServer<BlockHash> {
    //transfer to substrate address
    #[rpc(name = "transfer_to_substrate_account")]
    fn transfer_to_substrate_account(&self, parameter:  TransferParam<AccountId>);
}

pub struct TransferApi<C> {
    client: Arc<C>
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
    fn transfer_to_substrate_account(&self, param: TransferParam<AccountId>) {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);

        api.transfer_to_substrate_account(&at, param)
            .map_err(|e| Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "Transfer to substrate account failed.".into(),
                data: Some(format!("{:?}", e).into()),
            });
    }
}