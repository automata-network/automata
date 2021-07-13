//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

use automata_primitives::{Block, AccountId, Balance, Index, Hash};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::{Error as BlockChainError, HeaderMetadata, HeaderBackend};
use sp_block_builder::BlockBuilder;
pub use sc_rpc_api::DenyUnsafe;
use sp_transaction_pool::TransactionPool;
use sc_network::NetworkService;
use fc_rpc_core::types::PendingTransactions;
use jsonrpc_pubsub::manager::SubscriptionManager;
use sc_rpc::SubscriptionTaskExecutor;
use std::collections::BTreeMap;
use pallet_ethereum::EthereumStorageSchema;
use fc_rpc::{StorageOverride, SchemaV1Override};
use sc_client_api::{
	backend::{StorageProvider, Backend, StateBackend, AuxStore},
	client::BlockchainEvents
};
use sp_runtime::traits::BlakeTwo256;

/// Full client dependencies.
pub struct FullDeps<C, P> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
	/// Whether to enable dev signer
	pub enable_dev_signer: bool,
	/// Network service
	pub network: Arc<NetworkService<Block, Hash>>,
	/// Ethereum pending transactions.
    pub pending_transactions: PendingTransactions,
	/// The Node authority flag
	pub is_authority: bool,
	/// Backend.
	pub backend: Arc<fc_db::Backend<Block>>,
}

/// Instantiate all full RPC extensions.
pub fn create_full<C, P, BE>(
	deps: FullDeps<C, P>,
	subscription_task_executor: SubscriptionTaskExecutor,
) -> jsonrpc_core::IoHandler<sc_rpc::Metadata> where
	BE: Backend<Block> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
	C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + sc_client_api::AuxStore,
	C: BlockchainEvents<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error=BlockChainError> + 'static,
	C: Send + Sync + 'static,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
	C::Api: BlockBuilder<Block>,
	P: TransactionPool<Block=Block> + 'static,
{
    use fc_rpc::{
        EthApi, EthApiServer, EthDevSigner, EthPubSubApi, EthPubSubApiServer, EthSigner,
        HexEncodedIdProvider, NetApi, NetApiServer, Web3Api, Web3ApiServer,
    };

	use substrate_frame_rpc_system::{FullSystem, SystemApi};
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};

	let mut io = jsonrpc_core::IoHandler::default();
	let FullDeps {
		client,
		pool,
		deny_unsafe,
		enable_dev_signer,
		network,
		pending_transactions,
		is_authority,
		backend
	} = deps;

	io.extend_with(
		SystemApi::to_delegate(FullSystem::new(client.clone(), pool.clone(), deny_unsafe))
	);

	io.extend_with(
		TransactionPaymentApi::to_delegate(TransactionPayment::new(client.clone()))
	);

	// Extend this RPC with a custom API by using the following syntax.
	// `YourRpcStruct` should have a reference to a client, which is needed
	// to call into the runtime.
	// `io.extend_with(YourRpcTrait::to_delegate(YourRpcStruct::new(ReferenceToClient, ...)));`

    let mut signers = Vec::new();
    if enable_dev_signer {
        signers.push(Box::new(EthDevSigner::new()) as Box<dyn EthSigner>);
    }

	let mut overrides_map = BTreeMap::new();
	overrides_map.insert(
		EthereumStorageSchema::V1,
		Box::new(SchemaV1Override::new(client.clone())) as Box<dyn StorageOverride<_> + Send + Sync>
	);

	io.extend_with(EthApiServer::to_delegate(EthApi::new(
        client.clone(),
        pool.clone(),
        automata_runtime::TransactionConverter,
        network.clone(),
        pending_transactions.clone(),
        signers,
		overrides_map,
		backend,
        is_authority,
    )));

	io.extend_with(NetApiServer::to_delegate(NetApi::new(
        client.clone(),
        network.clone(),
    )));

	io.extend_with(Web3ApiServer::to_delegate(Web3Api::new(client.clone())));

	io.extend_with(EthPubSubApiServer::to_delegate(EthPubSubApi::new(
        pool.clone(),
        client.clone(),
        network.clone(),
        SubscriptionManager::<HexEncodedIdProvider>::with_id_provider(
            HexEncodedIdProvider::default(),
            Arc::new(subscription_task_executor),
        ),
    )));

	io
}
