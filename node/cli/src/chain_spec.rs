// SPDX-License-Identifier: Apache-2.0

use automata_runtime::constants::currency::*;
use automata_runtime::Block;
use automata_runtime::{
    wasm_binary_unwrap, AuraConfig, AuthorityDiscoveryConfig, BalancesConfig, CouncilConfig,
    DemocracyConfig, EVMConfig, ElectionsConfig, EthereumConfig, GrandpaConfig, ImOnlineConfig,
    IndicesConfig, SessionConfig, SessionKeys, StakerStatus, StakingConfig, SudoConfig,
    SystemConfig, TechnicalCommitteeConfig,
};
use grandpa_primitives::AuthorityId as GrandpaId;
use hex_literal::hex;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainSpecExtension;
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{
    crypto::{Ss58Codec, UncheckedInto},
    sr25519, Pair, Public,
};
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    Perbill,
};

pub use automata_primitives::{AccountId, Balance, Signature};
pub use automata_runtime::GenesisConfig;

use sp_core::{H160, U256};

type AccountPublic = <Signature as Verify>::Signer;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    /// Block numbers with known hashes.
    pub fork_blocks: sc_client_api::ForkBlocks<Block>,
    /// Known bad block hashes.
    pub bad_blocks: sc_client_api::BadBlocks<Block>,
}

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

fn get_properties() -> Option<Properties> {
    let mut properties = Properties::new();
    properties.insert("tokenSymbol".into(), "ATA".into());
    properties.insert("tokenDecimals".into(), 9.into());
    Some(properties)
}

fn session_keys(
    grandpa: GrandpaId,
    aura: AuraId,
    im_online: ImOnlineId,
    authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
    SessionKeys {
        grandpa,
        aura,
        im_online,
        authority_discovery,
    }
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(
    seed: &str,
) -> (
    AccountId,
    AccountId,
    GrandpaId,
    AuraId,
    ImOnlineId,
    AuthorityDiscoveryId,
) {
    (
        get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<AuraId>(seed),
        get_from_seed::<ImOnlineId>(seed),
        get_from_seed::<AuthorityDiscoveryId>(seed),
    )
}

/// Helper function to create GenesisConfig for testing
pub fn testnet_genesis(
    initial_authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        AuraId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )>,
    root_key: AccountId,
    endowed_accounts: Option<Vec<AccountId>>,
    ethereum_accounts: Option<Vec<AccountId>>,
    _enable_println: bool,
) -> GenesisConfig {
    let mut endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
        vec![
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            get_account_id_from_seed::<sr25519::Public>("Bob"),
            get_account_id_from_seed::<sr25519::Public>("Charlie"),
            get_account_id_from_seed::<sr25519::Public>("Dave"),
            get_account_id_from_seed::<sr25519::Public>("Eve"),
            get_account_id_from_seed::<sr25519::Public>("Ferdie"),
            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
            get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
            get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
            get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
            get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
            get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
        ]
    });

    initial_authorities.iter().for_each(|x| {
        if !endowed_accounts.contains(&x.0) {
            endowed_accounts.push(x.0.clone())
        }
    });

    if let Some(ethereum_accounts) = ethereum_accounts {
        ethereum_accounts.iter().for_each(|x| {
            if !endowed_accounts.contains(&x) {
                endowed_accounts.push(x.clone())
            }
        });
    }

    let num_endowed_accounts = endowed_accounts.len();
    const ENDOWMENT: Balance = 100_000_000 * DOLLARS;
    const STASH: Balance = 100 * DOLLARS;

    GenesisConfig {
        frame_system: Some(SystemConfig {
            code: wasm_binary_unwrap().to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_balances: Some(BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, ENDOWMENT))
                .collect(),
        }),
        pallet_indices: Some(IndicesConfig { indices: vec![] }),
        pallet_session: Some(SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),
                        x.0.clone(),
                        session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        }),
        pallet_staking: Some(StakingConfig {
            validator_count: initial_authorities.len() as u32 * 2,
            minimum_validator_count: initial_authorities.len() as u32,
            stakers: initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
                .collect(),
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: Perbill::from_percent(10),
            ..Default::default()
        }),
        pallet_democracy: Some(DemocracyConfig::default()),
        pallet_elections_phragmen: Some(ElectionsConfig {
            members: endowed_accounts
                .iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .map(|member| (member, STASH))
                .collect(),
        }),
        pallet_collective_Instance1: Some(CouncilConfig::default()),
        pallet_collective_Instance2: Some(TechnicalCommitteeConfig {
            members: endowed_accounts
                .iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .collect(),
            phantom: Default::default(),
        }),
        pallet_sudo: Some(SudoConfig { key: root_key }),
        pallet_aura: Some(AuraConfig {
            authorities: vec![],
        }),
        pallet_im_online: Some(ImOnlineConfig { keys: vec![] }),
        pallet_authority_discovery: Some(AuthorityDiscoveryConfig { keys: vec![] }),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: vec![],
        }),
        pallet_membership_Instance1: Some(Default::default()),
        pallet_evm: Some(EVMConfig {
            accounts: vec![
                H160::from(hex_literal::hex![
                    "18bD778c044F47d41CFabF336F2b1e06648e0771"
                ]),
                H160::from(hex_literal::hex![
                    "b4b58365166402a78b4ac05e1b13b6d64fCcF60f"
                ]),
                H160::from(hex_literal::hex![
                    "2CCDD9Fa13d97F6FAEC4B1D8085861AE57e1D9c9"
                ]),
                H160::from(hex_literal::hex![
                    "3e29eF30D9836928DDc3667af68da02bAd913316"
                ]),
            ]
            .into_iter()
            .map(|x| {
                (
                    x,
                    pallet_evm::GenesisAccount {
                        balance: U256::from(ENDOWMENT),
                        nonce: Default::default(),
                        code: Default::default(),
                        storage: Default::default(),
                    },
                )
            })
            .collect(),
        }),
        pallet_ethereum: Some(EthereumConfig {}),
    }
}

fn development_config_genesis() -> GenesisConfig {
    let accounts =
        vec![
            AccountId::from_ss58check("5ENPmNpr6TmsiCBY1MjFXn4pFzApNh3BVm1hF38ok9DVgQ6s").unwrap(),
        ];

    testnet_genesis(
        vec![authority_keys_from_seed("Alice")],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        None,
        Some(accounts),
        true,
    )
}

fn local_testnet_genesis() -> GenesisConfig {
    testnet_genesis(
        vec![
            authority_keys_from_seed("Alice"),
            authority_keys_from_seed("Bob"),
        ],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        None,
        None,
        false,
    )
}

fn staging_testnet_config_genesis() -> GenesisConfig {
    let ethereum_accounts =
        vec![
            AccountId::from_ss58check("5ENPmNpr6TmsiCBY1MjFXn4pFzApNh3BVm1hF38ok9DVgQ6s").unwrap(),
        ];

    // stash, controller, session-key
    // generated with secret:
    // for i in 1 2 3 4 ; do for j in stash controller; do subkey inspect "$secret"/automata/$j/$i; done; done
    // and
    // for i in 1 2 3 4 ; do for j in session; do subkey --ed25519 inspect "$secret"//automata//$j//$i; done; done

    let initial_authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        AuraId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )> = vec![
        (
            // 5EpoJPebo3FWWahy6i9dcNzjzyTNe1J5zwsQu5NEmg4Yr9PQ
            hex!["7a1996d0fc27b5a0b8c8292ab10e3311045f3bb9ee52353ba93060d0fe433076"].into(),
            // 5CACY4gkbkiKh2BWNHaqKMFBhreKFi119aokLTjwe6Wj2dha
            hex!["0430c51a3882a8d9d4d6ecc04f9058e96690aca93cb145b1e1b0b6a010222e0f"].into(),
            // 5FEQBJZuHQidTV2Y1PjJR2SipmDgtYc9PnULS39Fzg1JuMdF
            hex!["8c19a47f493eb8135b010ff1e12d3bf920f66de2d84d0104a409b3947204329c"]
                .unchecked_into(),
            // 5GzqxNzhJspoNAhT6AswEwbH6b5AsJumWPuxVRVLk3QLFnux
            hex!["da3b7291438a4d373623628d116f0407f6d957c5956e6da15cfb01d50a63777d"]
                .unchecked_into(),
            // 5GzqxNzhJspoNAhT6AswEwbH6b5AsJumWPuxVRVLk3QLFnux
            hex!["da3b7291438a4d373623628d116f0407f6d957c5956e6da15cfb01d50a63777d"]
                .unchecked_into(),
            // 5GzqxNzhJspoNAhT6AswEwbH6b5AsJumWPuxVRVLk3QLFnux
            hex!["da3b7291438a4d373623628d116f0407f6d957c5956e6da15cfb01d50a63777d"]
                .unchecked_into(),
        ),
        (
            // 5Cw8TCFEp6KJaEDh1zYYcguptz7NMDt4owqEBr9v1253Jv99
            hex!["267510b720ec2bbccd97c2f87a12c433d72dc3fb1febaae8307f462717ac7f32"].into(),
            // 5H1Cf9tqnLKDShc2fncftqZhNmHRSEbBH6SQxjScuBkG7wZc
            hex!["da8123da17c37a4af164516d9d5322ba7c806be0096d5112d929cb90ae875537"].into(),
            // 5EmXdcsvanPK8GtkgVN2s6n1jq3NchGmDunFx5quQu6b2sHA
            hex!["779b1bacb9605c61b2ac687ef835e2c99b6a5f8746f6549c109f22faddb8a100"]
                .unchecked_into(),
            // 5HatiikkNpBtfgyMXhrteSG2wUfasqcYS4xFdjzYarMU8y9W
            hex!["f43318eee81b201bbda6f3eea82736cbf39c80292e52dbfe2586504b80ce4137"]
                .unchecked_into(),
            // 5HatiikkNpBtfgyMXhrteSG2wUfasqcYS4xFdjzYarMU8y9W
            hex!["f43318eee81b201bbda6f3eea82736cbf39c80292e52dbfe2586504b80ce4137"]
                .unchecked_into(),
            // 5HatiikkNpBtfgyMXhrteSG2wUfasqcYS4xFdjzYarMU8y9W
            hex!["f43318eee81b201bbda6f3eea82736cbf39c80292e52dbfe2586504b80ce4137"]
                .unchecked_into(),
        ),
        (
            // 5HigvfW7JfXig7MXpCAhnu1quSWRU96CzaRDAoS4nFYuMTUT
            hex!["fa255bb650e8b3c88b74cd546b6f51d20764f8e207e6822e8c41a727d1176928"].into(),
            // 5FZPXATdvPQJoAwqsj4QaX36X4nGavRVXiEbDn1M4J8Gqoob
            hex!["9a951775099769f02bebcdceb66791566ec65d710ff87fc8845b1c51e9147f73"].into(),
            // 5Fw4twqFtEKvap9LUEBo8JgDYz9P5EvCNH7MpN9NYLRjhGhd
            hex!["ab1dcb89190c264527d9ba9645035fa6a7dbe71fa1ab28a0803f5fbc097a9c63"]
                .unchecked_into(),
            // 5DCSE99RB1YZ22ntanKrzgZsWiGLYY5LXXJU9iNPVWgC7oLw
            hex!["3221950754289dcba6ad9b2a84aee41b5cbe8aa72f64677ca01826b17580c95c"]
                .unchecked_into(),
            // 5DCSE99RB1YZ22ntanKrzgZsWiGLYY5LXXJU9iNPVWgC7oLw
            hex!["3221950754289dcba6ad9b2a84aee41b5cbe8aa72f64677ca01826b17580c95c"]
                .unchecked_into(),
            // 5DCSE99RB1YZ22ntanKrzgZsWiGLYY5LXXJU9iNPVWgC7oLw
            hex!["3221950754289dcba6ad9b2a84aee41b5cbe8aa72f64677ca01826b17580c95c"]
                .unchecked_into(),
        ),
        (
            // 5DPaPfAoEqwNvHtd3Wv34ZKqYoXLdpR95LPq8ZzAnHMGRv7q
            hex!["3aa0c3af40b7d3d428aac90fccc0687372688d01c1843744d59055d2f735a431"].into(),
            // 5FhmA2hoGahobxhDumWG7uJt81bzbucoyfjt7UMdWBmq5gcY
            hex!["a0f7e30a0ec26590ec3976fe3369ae3bdb326410813c79bf18a241bf1bd31d57"].into(),
            // 5GJFSUuHFWDC21ShQ8tvZG2n8UDF5xuY8dkc1G4yL31LnyiW
            hex!["bb45711e96f16cf7341d2dad00a5b177a1347b0131f56fa3f837e9dcb46a61d4"]
                .unchecked_into(),
            // 5CuC84fjuaLvZzoRXGsNNJHTUgPR8JjvqnrqNwfVjeTESNHK
            hex!["24faede9dc6c3eb99771d470de6ab76cd103fc8af275e33ce38ca887ee49e97a"]
                .unchecked_into(),
            // 5CuC84fjuaLvZzoRXGsNNJHTUgPR8JjvqnrqNwfVjeTESNHK
            hex!["24faede9dc6c3eb99771d470de6ab76cd103fc8af275e33ce38ca887ee49e97a"]
                .unchecked_into(),
            // 5CuC84fjuaLvZzoRXGsNNJHTUgPR8JjvqnrqNwfVjeTESNHK
            hex!["24faede9dc6c3eb99771d470de6ab76cd103fc8af275e33ce38ca887ee49e97a"]
                .unchecked_into(),
        ),
    ];

    // generated with secret: subkey inspect "$secret"/automata
    let root_key: AccountId = hex![
        // 5HGWsrBrgNxVTisu5DYjfGGbCf69VxtybSW8t36arFUabVtn
        "e62f26bc433a9fa7679a284b1f85898739c32ab4b23246515be0ee339643003f"
    ]
    .into();

    let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

    testnet_genesis(
        initial_authorities,
        root_key,
        Some(endowed_accounts),
        Some(ethereum_accounts),
        false,
    )
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Development",
        "dev",
        ChainType::Development,
        development_config_genesis,
        vec![],
        None,
        None,
        get_properties(),
        Default::default(),
    )
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn local_testnet_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Local Testnet",
        "local_testnet",
        ChainType::Local,
        local_testnet_genesis,
        vec![],
        None,
        None,
        get_properties(),
        Default::default(),
    )
}

/// Staging testnet config.
pub fn staging_testnet_config() -> ChainSpec {
    let boot_nodes = vec![];
    ChainSpec::from_genesis(
        "Staging Testnet",
        "staging_testnet",
        ChainType::Live,
        staging_testnet_config_genesis,
        boot_nodes,
        Some(
            TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Staging telemetry url is valid; qed"),
        ),
        None,
        get_properties(),
        Default::default(),
    )
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::service::{new_full_base, new_light_base, NewFullBase};
    use sc_service_test;
    use sp_runtime::BuildStorage;

    fn local_testnet_genesis_instant_single() -> GenesisConfig {
        testnet_genesis(
            vec![authority_keys_from_seed("Alice")],
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            None,
            None,
            false,
        )
    }

    /// Local testnet config (single validator - Alice)
    pub fn integration_test_config_with_single_authority() -> ChainSpec {
        ChainSpec::from_genesis(
            "Integration Test",
            "test",
            ChainType::Development,
            local_testnet_genesis_instant_single,
            vec![],
            None,
            None,
            get_properties(),
            Default::default(),
        )
    }

    /// Local testnet config (multivalidator Alice + Bob)
    pub fn integration_test_config_with_two_authorities() -> ChainSpec {
        ChainSpec::from_genesis(
            "Integration Test",
            "test",
            ChainType::Development,
            local_testnet_genesis,
            vec![],
            None,
            None,
            get_properties(),
            Default::default(),
        )
    }

    #[test]
    #[ignore]
    fn test_connectivity() {
        sc_service_test::connectivity(
            integration_test_config_with_two_authorities(),
            |config| {
                let NewFullBase {
                    task_manager,
                    client,
                    network,
                    transaction_pool,
                    ..
                } = new_full_base(config)?;
                Ok(sc_service_test::TestNetComponents::new(
                    task_manager,
                    client,
                    network,
                    transaction_pool,
                ))
            },
            |config| {
                let (keep_alive, _, _, client, network, transaction_pool) = new_light_base(config)?;
                Ok(sc_service_test::TestNetComponents::new(
                    keep_alive,
                    client,
                    network,
                    transaction_pool,
                ))
            },
        );
    }

    #[test]
    fn test_create_development_chain_spec() {
        development_config().build_storage().unwrap();
    }

    #[test]
    fn test_create_local_testnet_chain_spec() {
        local_testnet_config().build_storage().unwrap();
    }

    #[test]
    fn test_staging_test_net_chain_spec() {
        staging_testnet_config().build_storage().unwrap();
    }
}
