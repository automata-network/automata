use automata_primitives::{AccountId, Block, BlockId, Index};
use automata_runtime::apis::TransferApi as TransferRuntimeApi;
use fp_rpc::EthereumRuntimeRPCApi;
use frame_system_rpc_runtime_api::AccountNonceApi;
use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;
use pallet_transfer::{eth_recover, evm_address_to_account_id_bytes};
use sc_light::blockchain::BlockchainHeaderBackend as HeaderBackend;
use sp_api::ProvideRuntimeApi;
use sp_core::ecdsa;
use sp_runtime::{codec::Decode, traits::Block as BlockT};
use std::sync::Arc;

const RUNTIME_ERROR: i64 = 1;

#[rpc]
pub trait TransferServer<BlockHash> {
    //transfer to substrate address
    #[rpc(name = "transfer_transferToSubstrateAccount")]
    fn transfer_to_substrate_account(&self, message: String, signature: String) -> Result<u64>;

    #[rpc(name = "transfer_transferNonce")]
    fn transfer_nonce(&self, evm_addr: String) -> Result<u32>;
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
    C::Api: TransferRuntimeApi<Block>
        + EthereumRuntimeRPCApi<Block>
        + AccountNonceApi<Block, AccountId, Index>,
{
    fn transfer_to_substrate_account(&self, message: String, signature: String) -> Result<u64> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);

        let mut signature_bytes = [0u8; 65];
        let signature_param_bytes = match hex::decode(&signature) {
            Ok(bytes) => bytes,
            Err(e) => {
                return Err(Error {
                    code: ErrorCode::ServerError(RUNTIME_ERROR),
                    message: "Failed to decode signature.".into(),
                    data: Some(format!("{:?}", e).into()),
                })
            }
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

        let mut message_bytes = [0u8; 72];
        let message_param_bytes = match hex::decode(message) {
            Ok(bytes) => bytes,
            Err(e) => {
                return Err(Error {
                    code: ErrorCode::ServerError(RUNTIME_ERROR),
                    message: "Failed to decode message.".into(),
                    data: Some(format!("{:?}", e).into()),
                })
            }
        };
        if message_param_bytes.len() != 72 {
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
        let source_account_id_bytes = evm_address_to_account_id_bytes(source_address);
        let source_account_id =
            AccountId::decode(&mut &source_account_id_bytes[..]).unwrap_or_default();

        //transfer amount: 52-67 bytes
        let mut value_bytes = [0u8; 16];
        value_bytes.copy_from_slice(&message_bytes[52..68]);
        let value_128: u128 = u128::from_be_bytes(value_bytes);

        //nonce: 68-71
        let mut nonce_bytes = [0u8; 4];
        nonce_bytes.copy_from_slice(&message_bytes[68..72]);
        let nonce: Index = u32::from_be_bytes(nonce_bytes).into();

        let address = match eth_recover(&signature, &message_bytes, &[][..]) {
            Some(addr) => addr,
            None => {
                return Err(Error {
                    code: ErrorCode::ServerError(RUNTIME_ERROR),
                    message: "Failed to recover message signer.".into(),
                    data: None,
                })
            }
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
        //make sure use correct nonce
        let real_nonce = match api.account_nonce(&at, source_account_id) {
            Ok(nonce) => nonce,
            Err(e) => {
                return Err(Error {
                    code: ErrorCode::ServerError(RUNTIME_ERROR),
                    message: "Failed to get nonce.".into(),
                    data: Some(format!("{:?}", e).into()),
                })
            }
        };
        if nonce != real_nonce {
            return Err(Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "Account nonce mismatch.".into(),
                data: None,
            });
        }

        //submit a unsigned extrinsics into transaction pool
        let _ = api
            .submit_unsigned_transaction(&at, message_bytes, signature_bytes)
            .map_err(|e| Error {
                code: ErrorCode::ServerError(RUNTIME_ERROR),
                message: "Failed to submit unsigned extrinsics.".into(),
                data: Some(format!("{:?}", e).into()),
            });

        //TODO how to handle result???

        Ok(0)
    }

    fn transfer_nonce(&self, evm_addr: String) -> Result<u32> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);

        let evm_addr_bytes = match hex::decode(&evm_addr) {
            Ok(bytes) => bytes,
            Err(e) => {
                return Err(Error {
                    code: ErrorCode::ServerError(RUNTIME_ERROR),
                    message: "Failed to decode address.".into(),
                    data: Some(format!("{:?}", e).into()),
                })
            }
        };
        let mut address_bytes = [0u8; 20];
        address_bytes.copy_from_slice(&evm_addr_bytes[..]);
        let evm_address = address_bytes.into();
        let account_id_bytes = evm_address_to_account_id_bytes(evm_address);
        let account_id = AccountId::decode(&mut &account_id_bytes[..]).unwrap_or_default();

        let nonce = match api.account_nonce(&at, account_id) {
            Ok(nonce) => nonce,
            Err(e) => {
                return Err(Error {
                    code: ErrorCode::ServerError(RUNTIME_ERROR),
                    message: "Failed to get nonce.".into(),
                    data: Some(format!("{:?}", e).into()),
                })
            }
        };

        Ok(nonce)
    }
}
