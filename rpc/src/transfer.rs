use automata_primitives::{Block, BlockId};
use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sc_light::blockchain::BlockchainHeaderBackend as HeaderBackend;
use sp_api::ProvideRuntimeApi;
use sp_runtime::{traits::Block as BlockT};
use std::sync::Arc;
use sp_core::ecdsa;
use automata_runtime::apis::TransferApi as TransferRuntimeApi;
use fp_rpc::EthereumRuntimeRPCApi;
use pallet_transfer::{eth_recover};

const RUNTIME_ERROR: i64 = 1;

#[rpc]
pub trait TransferServer<BlockHash> {
    //transfer to substrate address
    #[rpc(name = "transfer_to_substrate_account")]
    fn transfer_to_substrate_account(
        &self, 
        message: String,
        signature: String
    ) -> Result<u64>;
}

pub struct TransferApi<C> {
    client: Arc<C>,
}

impl<C> TransferApi<C> {
    pub fn new(client: Arc<C>) -> Self {
        TransferApi { 
            client,
        }
    }
}

impl<C> TransferServer<<Block as BlockT>::Hash> for TransferApi<C>
where
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: TransferRuntimeApi<Block> + EthereumRuntimeRPCApi<Block>,
{
    fn transfer_to_substrate_account(
        &self, 
        message: String,
        signature: String
    ) -> Result<u64> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);

        let mut signature_bytes = [0u8; 65];
        let signature_param_bytes = match hex::decode(&signature) {
            Ok(bytes) => bytes,
            Err(e) => return Err(Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "Failed to decode signature.".into(),
                data: Some(format!("{:?}", e).into()),
            }),
        };
        if signature_param_bytes.len() != 65 {
            return Err(Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "Signature bytes length should be 65.".into(),
                data: None,
            });
        }
        signature_bytes.copy_from_slice(&signature_param_bytes);
        let signature = ecdsa::Signature::from_slice(&signature_bytes);

        let mut message_bytes = [0u8; 68];
        let message_param_bytes = match hex::decode(message) {
            Ok(bytes) => bytes,
            Err(e) => return Err(Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "Failed to decode message.".into(),
                data: Some(format!("{:?}", e).into()),
            }),
        };
        if message_param_bytes.len() != 68 {
            return Err(Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "Message bytes length should be 65.".into(),
                data: None,
            });
        }
        message_bytes.copy_from_slice(&message_param_bytes);
        //source address bytes(evm): 0-19 bytes
        let mut source_address_bytes = [0u8; 20];
        source_address_bytes.copy_from_slice(&message_bytes[0..20]);
        let source_address = source_address_bytes.into();

        //transfer amount: 52-68 bytes
        let mut value_bytes = [0u8; 16];
        value_bytes.copy_from_slice(&message_bytes[52..68]);
        let value_128: u128 = u128::from_be_bytes(value_bytes);

        let address = match eth_recover(&signature, &message_bytes, &[][..]) {
            Some(addr) => addr,
            None => return Err(Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "Failed to recover message signer.".into(),
                data: None,
            }),
        };

        //make sure that the signature is signed by source_address
        if address != source_address {
            return Err(Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "Source address mismatch.".into(),
                data: None,
            });
        }
        //make sure source address has sufficient balance???
        let account_basic = api.account_basic(&at, source_address).unwrap();
        if account_basic.balance < value_128.into() {
            return Err(Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "Insufficient balance.".into(),
                data: None,
            });
        }

        //submit a unsigned extrinsics into transaction pool
        let _ = api.submit_unsigned_transaction(&at, message_bytes, signature_bytes).map_err(|e| Error {
            code: ErrorCode::ServerError(RUNTIME_ERROR),
            message: "Failed to submit unsigned extrinsics.".into(),
            data: Some(format!("{:?}", e).into()),
        });

        //TODO how to handle result???

        Ok(0)
    }
}
