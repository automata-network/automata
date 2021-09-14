//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

use automata_primitives::{AccountId, Balance, Block, BlockNumber, Hash, Index};
use automata_runtime::apis::AttestorApi as AttestorRuntimeApi;
use automata_runtime::apis::GeodeApi as GeodeRuntimeApi;
use automata_runtime::apis::TransferApi as TransferRuntimeApi;
use fc_rpc::{SchemaV1Override, StorageOverride};
use fc_rpc_core::types::PendingTransactions;
use jsonrpc_pubsub::manager::SubscriptionManager;
use pallet_ethereum::EthereumStorageSchema;
use sc_client_api::{
    backend::{Backend, StateBackend, StorageProvider},
    client::BlockchainEvents,
};
use sc_consensus_babe::{Config, Epoch};
use sc_consensus_epochs::SharedEpochChanges;
use sc_finality_grandpa::{
    FinalityProofProvider, GrandpaJustificationStream, SharedAuthoritySet, SharedVoterState,
};
use sc_network::NetworkService;
use sc_rpc::SubscriptionTaskExecutor;
pub use sc_rpc_api::DenyUnsafe;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_keystore::SyncCryptoStorePtr;
use sp_runtime::traits::BlakeTwo256;
use sp_transaction_pool::TransactionPool;
use std::collections::BTreeMap;
use std::sync::Arc;

pub mod attestor;
pub mod geode;
pub mod transfer;

/// Extra dependencies for BABE.
pub struct BabeDeps {
    /// BABE protocol config.
    pub babe_config: Config,
    /// BABE pending epoch changes.
    pub shared_epoch_changes: SharedEpochChanges<Block, Epoch>,
    /// The keystore that manages the keys of the node.
    pub keystore: SyncCryptoStorePtr,
}

/// Extra dependencies for GRANDPA
pub struct GrandpaDeps<B> {
    /// Voting round info.
    pub shared_voter_state: SharedVoterState,
    /// Authority set info.
    pub shared_authority_set: SharedAuthoritySet<Hash, BlockNumber>,
    /// Receives notifications about justification events from Grandpa.
    pub justification_stream: GrandpaJustificationStream<Block>,
    /// Subscription manager to keep track of pubsub subscribers.
    pub subscription_executor: SubscriptionTaskExecutor,
    /// Finality proof provider.
    pub finality_provider: Arc<FinalityProofProvider<B, Block>>,
}

/// Full client dependencies.
pub struct FullDeps<C, P, B, SC> {
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
    /// The SelectChain Strategy
    pub select_chain: SC,
    /// BABE specific dependencies.
    pub babe: BabeDeps,
    /// GRANDPA specific dependencies.
    pub grandpa: GrandpaDeps<B>,
}

/// Instantiate all full RPC extensions.
pub fn create_full<C, P, BE, B, SC>(
    deps: FullDeps<C, P, B, SC>,
    subscription_task_executor: SubscriptionTaskExecutor,
) -> jsonrpc_core::IoHandler<sc_rpc::Metadata>
where
    BE: Backend<Block> + 'static,
    BE::State: StateBackend<BlakeTwo256>,
    C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + sc_client_api::AuxStore,
    C: BlockchainEvents<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
    C::Api: sp_consensus_babe::BabeApi<Block>,
    C::Api: BlockBuilder<Block>,
    C::Api: AttestorRuntimeApi<Block>,
    C::Api: GeodeRuntimeApi<Block>,
    C::Api: TransferRuntimeApi<Block>,
    P: TransactionPool<Block = Block> + 'static,
    B: sc_client_api::Backend<Block> + Send + Sync + 'static,
    B::State: sc_client_api::StateBackend<sp_runtime::traits::HashFor<Block>>,
    SC: sp_consensus::SelectChain<Block> + 'static,
{
    use fc_rpc::{
        EthApi, EthApiServer, EthDevSigner, EthPubSubApi, EthPubSubApiServer, EthSigner,
        HexEncodedIdProvider, NetApi, NetApiServer, Web3Api, Web3ApiServer,
    };

    use attestor::AttestorServer;
    use geode::GeodeServer;
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
    use sc_consensus_babe_rpc::BabeRpcHandler;
    use sc_finality_grandpa_rpc::GrandpaRpcHandler;
    use substrate_frame_rpc_system::{FullSystem, SystemApi};
    use transfer::TransferServer;

    let mut io = jsonrpc_core::IoHandler::default();
    let FullDeps {
        client,
        pool,
        deny_unsafe,
        enable_dev_signer,
        network,
        pending_transactions,
        is_authority,
        backend,
        select_chain,
        babe,
        grandpa,
    } = deps;

    let BabeDeps {
        keystore,
        babe_config,
        shared_epoch_changes,
    } = babe;

    let GrandpaDeps {
        shared_voter_state,
        shared_authority_set,
        justification_stream,
        subscription_executor,
        finality_provider,
    } = grandpa;

    io.extend_with(SystemApi::to_delegate(FullSystem::new(
        client.clone(),
        pool.clone(),
        deny_unsafe,
    )));

    io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
        client.clone(),
    )));

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
        Box::new(SchemaV1Override::new(client.clone()))
            as Box<dyn StorageOverride<_> + Send + Sync>,
    );

    io.extend_with(EthApiServer::to_delegate(EthApi::new(
        client.clone(),
        pool.clone(),
        automata_runtime::TransactionConverter,
        network.clone(),
        pending_transactions,
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

    io.extend_with(sc_consensus_babe_rpc::BabeApi::to_delegate(
        BabeRpcHandler::new(
            client.clone(),
            shared_epoch_changes,
            keystore,
            babe_config,
            select_chain,
            deny_unsafe,
        ),
    ));

    io.extend_with(sc_finality_grandpa_rpc::GrandpaApi::to_delegate(
        GrandpaRpcHandler::new(
            shared_authority_set,
            shared_voter_state,
            justification_stream,
            subscription_executor,
            finality_provider,
        ),
    ));

    io.extend_with(EthPubSubApiServer::to_delegate(EthPubSubApi::new(
        pool.clone(),
        client.clone(),
        network,
        SubscriptionManager::<HexEncodedIdProvider>::with_id_provider(
            HexEncodedIdProvider::default(),
            Arc::new(subscription_task_executor.clone()),
        ),
    )));

    io.extend_with(AttestorServer::to_delegate(attestor::AttestorApi::new(
        client.clone(),
    )));

    io.extend_with(GeodeServer::to_delegate(geode::GeodeApi::new(
        client.clone(),
        SubscriptionManager::new(Arc::new(subscription_task_executor)),
    )));

    io.extend_with(TransferServer::to_delegate(transfer::TransferApi::new(
        client.clone(),
    )));

    io
}
