//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.
#[cfg(all(feature = "automata", feature = "contextfree"))]
compile_error!("Feature 1 and 2 are mutually exclusive and cannot be enabled together");
#[cfg(all(feature = "automata", feature = "finitestate"))]
compile_error!("Feature 1 and 2 are mutually exclusive and cannot be enabled together");
#[cfg(all(feature = "finitestate", feature = "contextfree"))]
compile_error!("Feature 1 and 2 are mutually exclusive and cannot be enabled together");

use automata_primitives::{AccountId, Balance, Block, BlockNumber, Hash, Index};
// #[cfg(feature = "automata")]
// use automata_runtime::apis::{
//     AttestorApi as AttestorRuntimeApi, GeodeApi as GeodeRuntimeApi,
//     TransferApi as TransferRuntimeApi,
// };
// #[cfg(feature = "contextfree")]
// use contextfree_runtime::apis::TransferApi as TransferRuntimeApi;
use fc_rpc::{OverrideHandle, RuntimeApiStorageOverride, SchemaV1Override, StorageOverride};
use fc_rpc_core::types::PendingTransactions;
// #[cfg(feature = "finitestate")]
// use finitestate_runtime::apis::TransferApi as TransferRuntimeApi;
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
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_keystore::SyncCryptoStorePtr;
use sp_runtime::traits::BlakeTwo256;
use std::collections::BTreeMap;
use std::sync::Arc;

// #[cfg(feature = "automata")]
// pub mod attestor;
// #[cfg(feature = "automata")]
// pub mod geode;
// #[cfg(feature = "automata")]
// pub mod transfer;

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
    /// Maximum number of logs in a query.
    pub max_past_logs: u32,
}

/// Instantiate all full RPC extensions.
#[cfg(any(feature = "contextfree", feature = "finitestate"))]
pub fn create_full<C, P, BE, B, SC>(
    deps: FullDeps<C, P, B, SC>,
    subscription_task_executor: SubscriptionTaskExecutor,
) -> Result<jsonrpc_core::IoHandler<sc_rpc::Metadata>, Box<dyn std::error::Error + Send + Sync>>
where
    BE: Backend<Block> + 'static,
    BE::State: StateBackend<BlakeTwo256>,
    C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + sc_client_api::backend::AuxStore,
    C: BlockchainEvents<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    // C::Api: RuntimeApis,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
    C::Api: sp_consensus_babe::BabeApi<Block>,
    C::Api: BlockBuilder<Block>,
    P: TransactionPool<Block = Block> + 'static,
    B: sc_client_api::Backend<Block> + Send + Sync + 'static,
    B::State: sc_client_api::StateBackend<sp_runtime::traits::HashFor<Block>>,
    SC: sp_consensus::SelectChain<Block> + 'static,
{
    Ok(create_full_base::<C, P, BE, B, SC>(
        deps,
        subscription_task_executor,
    ))
}

#[cfg(feature = "automata")]
pub fn create_full<C, P, BE, B, SC>(
    deps: FullDeps<C, P, B, SC>,
    subscription_task_executor: SubscriptionTaskExecutor,
) -> Result<jsonrpc_core::IoHandler<sc_rpc::Metadata>, Box<dyn std::error::Error + Send + Sync>>
where
    BE: Backend<Block> + 'static,
    BE::State: StateBackend<BlakeTwo256>,
    C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + sc_client_api::backend::AuxStore,
    C: BlockchainEvents<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    // C::Api: RuntimeApis,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
    C::Api: sp_consensus_babe::BabeApi<Block>,
    C::Api: BlockBuilder<Block>,
    // C::Api: AttestorRuntimeApi<Block>,
    // C::Api: GeodeRuntimeApi<Block>,
    // C::Api: TransferRuntimeApi<Block>,
    P: TransactionPool<Block = Block> + 'static,
    B: sc_client_api::Backend<Block> + Send + Sync + 'static,
    B::State: sc_client_api::StateBackend<sp_runtime::traits::HashFor<Block>>,
    SC: sp_consensus::SelectChain<Block> + 'static,
{
    // use transfer::TransferServer;

    let _client = deps.client.clone();
    let io = create_full_base::<C, P, BE, B, SC>(deps, subscription_task_executor);

    // io.extend_with(attestor::AttestorServer::to_delegate(
    //     attestor::AttestorApi::new(client.clone()),
    // ));

    // io.extend_with(geode::GeodeServer::to_delegate(geode::GeodeApi::new(
    //     client.clone(),
    // )));

    // io.extend_with(TransferServer::to_delegate(transfer::TransferApi::new(
    //     client.clone(),
    // )));

    Ok(io)
}

pub fn create_full_base<C, P, BE, B, SC>(
    deps: FullDeps<C, P, B, SC>,
    subscription_task_executor: SubscriptionTaskExecutor,
) -> jsonrpc_core::IoHandler<sc_rpc::Metadata>
where
    BE: Backend<Block> + 'static,
    BE::State: StateBackend<BlakeTwo256>,
    C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + sc_client_api::backend::AuxStore,
    C: BlockchainEvents<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    // C::Api: RuntimeApis,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
    C::Api: sp_consensus_babe::BabeApi<Block>,
    C::Api: BlockBuilder<Block>,
    // C::Api: AttestorRuntimeApi<Block>,
    // C::Api: GeodeRuntimeApi<Block>,
    // C::Api: TransferRuntimeApi<Block>,
    P: TransactionPool<Block = Block> + 'static,
    B: sc_client_api::Backend<Block> + Send + Sync + 'static,
    B::State: sc_client_api::StateBackend<sp_runtime::traits::HashFor<Block>>,
    SC: sp_consensus::SelectChain<Block> + 'static,
{
    use fc_rpc::{
        EthApi, EthApiServer, EthDevSigner, EthPubSubApi, EthPubSubApiServer, EthSigner,
        HexEncodedIdProvider, NetApi, NetApiServer, Web3Api, Web3ApiServer,
    };

    // #[cfg(feature = "automata")]
    // use attestor::AttestorServer;
    // #[cfg(feature = "automata")]
    // use geode::GeodeServer;
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
    use sc_consensus_babe_rpc::BabeRpcHandler;
    use sc_finality_grandpa_rpc::GrandpaRpcHandler;
    use substrate_frame_rpc_system::{FullSystem, SystemApi};

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
        max_past_logs,
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

    let overrides = Arc::new(OverrideHandle {
        schemas: overrides_map,
        fallback: Box::new(RuntimeApiStorageOverride::new(client.clone())),
    });

    io.extend_with(EthApiServer::to_delegate(EthApi::new(
        client.clone(),
        pool.clone(),
        #[cfg(feature = "automata")]
        automata_runtime::TransactionConverter,
        #[cfg(feature = "contextfree")]
        contextfree_runtime::TransactionConverter,
        #[cfg(feature = "finitestate")]
        finitestate_runtime::TransactionConverter,
        network.clone(),
        pending_transactions,
        signers,
        overrides.clone(),
        backend,
        is_authority,
        max_past_logs,
    )));

    io.extend_with(NetApiServer::to_delegate(NetApi::new(
        client.clone(),
        network.clone(),
        true,
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
            Arc::new(subscription_task_executor),
        ),
        overrides,
    )));

    io
}
