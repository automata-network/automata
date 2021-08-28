pub use automata_primitives::{AccountId, Balance, Signature};
use automata_runtime::constants::currency::*;
use automata_runtime::Block;
use automata_runtime::{
    opaque::SessionKeys, BabeConfig, BalancesConfig, EVMConfig, EthereumConfig, GenesisConfig,
    GrandpaConfig, IndicesConfig, SessionConfig, StakerStatus, StakingConfig, SudoConfig,
    ImOnlineConfig, SystemConfig, WASM_BINARY,
};
use hex_literal::hex;
use sc_chain_spec::ChainSpecExtension;
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{
    crypto::{Ss58Codec, UncheckedInto},
    sr25519, Pair, Public, H160, U256,
};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;

// The URL for the telemetry server.
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

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

fn get_properties() -> Option<Properties> {
    let mut properties = Properties::new();
    properties.insert("tokenSymbol".into(), "ATA".into());
    properties.insert("tokenDecimals".into(), 18.into());
    Some(properties)
}

fn get_session_keys(grandpa: GrandpaId, babe: BabeId, im_online: ImOnlineId) -> SessionKeys {
    SessionKeys { babe, grandpa, im_online }
}

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an authority key.
pub fn authority_keys_from_seed(s: &str) -> (AccountId, AccountId, GrandpaId, BabeId, ImOnlineId) {
    (
        get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", s)),
        get_account_id_from_seed::<sr25519::Public>(s),
        get_from_seed::<GrandpaId>(s),
        get_from_seed::<BabeId>(s),
        get_from_seed::<ImOnlineId>(s),
    )
}

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![authority_keys_from_seed("Alice")],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                None,
                Some(vec![AccountId::from_ss58check(
                    "5ENPmNpr6TmsiCBY1MjFXn4pFzApNh3BVm1hF38ok9DVgQ6s",
                )
                .unwrap()]),
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        get_properties(),
        // Extensions
        Default::default(),
    ))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![
                    authority_keys_from_seed("Alice"),
                    authority_keys_from_seed("Bob"),
                ],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                None,
                None,
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        get_properties(),
        // Extensions
        Default::default(),
    ))
}

/// Staging testnet config.
pub fn staging_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

    let boot_nodes = vec![];

    let initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId)> = vec![
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
        ),
    ];

    // generated with secret: subkey inspect "$secret"/automata
    let root_key: AccountId = hex![
        // 5HGWsrBrgNxVTisu5DYjfGGbCf69VxtybSW8t36arFUabVtn
        "e62f26bc433a9fa7679a284b1f85898739c32ab4b23246515be0ee339643003f"
    ]
    .into();

    let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

    let ethereum_accounts =
        vec![
            AccountId::from_ss58check("5ENPmNpr6TmsiCBY1MjFXn4pFzApNh3BVm1hF38ok9DVgQ6s").unwrap(),
        ];

    Ok(ChainSpec::from_genesis(
        "Staging Testnet",
        "staging_testnet",
        ChainType::Live,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                initial_authorities.clone(),
                // Sudo account
                root_key.clone(),
                // Pre-funded accounts
                Some(endowed_accounts.clone()),
                Some(ethereum_accounts.clone()),
                true,
            )
        },
        boot_nodes,
        Some(
            TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Staging telemetry url is valid; qed"),
        ),
        None,
        get_properties(),
        Default::default(),
    ))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId)>,
    root_key: AccountId,
    endowed_accounts: Option<Vec<AccountId>>,
    ethereum_accounts: Option<Vec<AccountId>>,
    _enable_println: bool,
) -> GenesisConfig {
    const INITIAL_STAKING: u128 = 1_000_000 * DOLLARS;
    const ENDOWMENT: Balance = 100_000_000 * DOLLARS;

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
            if !endowed_accounts.contains(x) {
                endowed_accounts.push(x.clone())
            }
        });
    }

    GenesisConfig {
        frame_system: Some(SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_balances: Some(BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
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
                        x.0.clone(), // stash
                        x.0.clone(), // stash
                        get_session_keys(
                            x.2.clone(), // grandpa
                            x.3.clone(), // babe
                            x.4.clone(),
                        ),
                    )
                })
                .collect::<Vec<_>>(),
        }),
        pallet_staking: Some(StakingConfig {
            validator_count: initial_authorities.len() as u32 * 2,
            minimum_validator_count: initial_authorities.len() as u32,
            stakers: initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),
                        x.1.clone(),
                        INITIAL_STAKING,
                        StakerStatus::Validator,
                    )
                })
                .collect(),
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: sp_runtime::Perbill::from_percent(10),
            ..Default::default()
        }),
        pallet_babe: Some(BabeConfig {
            authorities: vec![],
        }),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: vec![],
        }),
        pallet_im_online: Some(ImOnlineConfig {
            keys: vec![]
        }),
        // pallet_grandpa: Some(GrandpaConfig {
        //     authorities: initial_authorities
        //         .iter()
        //         .map(|x| (x.2.clone(), 1))
        //         .collect(),
        // }),
        pallet_sudo: Some(SudoConfig {
            // Assign network admin rights.
            key: root_key,
        }),
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
