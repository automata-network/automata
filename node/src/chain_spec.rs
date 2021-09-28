#[cfg(all(feature = "automata", feature = "contextfree"))]
compile_error!("Feature 1 and 2 are mutually exclusive and cannot be enabled together");

use automata_primitives::Block;
pub use automata_primitives::{AccountId, Balance, BlockNumber, Signature};
#[cfg(feature = "automata")]
use automata_runtime::{
    constants::currency::*, opaque::SessionKeys, AuthorityDiscoveryConfig, BabeConfig,
    BalancesConfig, EVMConfig, EthereumConfig, GenesisConfig, GrandpaConfig, ImOnlineConfig,
    IndicesConfig, SessionConfig, StakerStatus, StakingConfig, SudoConfig, SystemConfig,
    BABE_GENESIS_EPOCH_CONFIG, WASM_BINARY,
};
#[cfg(feature = "contextfree")]
use contextfree_runtime as contextfree;
#[cfg(feature = "contextfree")]
use contextfree_runtime::{constants::currency::*, GenesisConfig, StakerStatus};
use frame_support::PalletId;
use hex_literal::hex;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainSpecExtension;
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{
    crypto::{Ss58Codec, UncheckedInto},
    sr25519, Pair, Public, H160, U256,
};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{AccountIdConversion, IdentifyAccount, Verify};

#[cfg(feature = "contextfree")]
pub type ContextFreeChainSpec =
    sc_service::GenericChainSpec<contextfree::GenesisConfig, Extensions>;

// The URL for the telemetry server.
const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const DEFAULT_PROTOCOL_ID: &str = "ata";

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

#[cfg(feature = "automata")]
fn get_properties() -> Option<Properties> {
    let mut properties = Properties::new();
    properties.insert("tokenSymbol".into(), "ATA".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("ss58Format".into(), 2349.into());
    Some(properties)
}

#[cfg(feature = "contextfree")]
fn get_properties() -> Option<Properties> {
    let mut properties = Properties::new();
    properties.insert("tokenSymbol".into(), "CTX".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("ss58Format".into(), 11820.into());
    Some(properties)
}

#[cfg(feature = "contextfree")]
pub fn contextfree_chain_spec() -> Result<ContextFreeChainSpec, String> {
    ContextFreeChainSpec::from_json_bytes(
        &include_bytes!("../../assets/chain_spec_contextfree.json")[..],
    )
}

#[cfg(feature = "automata")]
fn get_session_keys(
    grandpa: GrandpaId,
    babe: BabeId,
    im_online: ImOnlineId,
    authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
    SessionKeys {
        babe,
        grandpa,
        im_online,
        authority_discovery,
    }
}

#[cfg(feature = "contextfree")]
fn get_contextfree_session_keys(
    grandpa: GrandpaId,
    babe: BabeId,
    im_online: ImOnlineId,
    authority_discovery: AuthorityDiscoveryId,
) -> contextfree::opaque::SessionKeys {
    contextfree::opaque::SessionKeys {
        babe,
        grandpa,
        im_online,
        authority_discovery,
    }
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
pub fn authority_keys_from_seed(
    s: &str,
) -> (
    AccountId,
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
) {
    (
        get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", s)),
        get_account_id_from_seed::<sr25519::Public>(s),
        get_from_seed::<GrandpaId>(s),
        get_from_seed::<BabeId>(s),
        get_from_seed::<ImOnlineId>(s),
        get_from_seed::<AuthorityDiscoveryId>(s),
    )
}

#[cfg(feature = "automata")]
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

#[cfg(feature = "automata")]
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

#[cfg(feature = "contextfree")]
pub fn contextfree_testnet_config() -> Result<ContextFreeChainSpec, String> {
    let wasm_binary = contextfree::WASM_BINARY.ok_or("ContextFree testnet awsm not available")?;
    let boot_nodes = vec![];

    Ok(ContextFreeChainSpec::from_genesis(
        "ContextFree Network",
        "contextfree_network",
        ChainType::Live,
        move || contextfree_config_genesis(wasm_binary),
        boot_nodes,
        Some(
            TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Staging telemetry url is valid; qed"),
        ),
        Some(DEFAULT_PROTOCOL_ID),
        get_properties(),
        Default::default(),
    ))
}

/// Staging testnet config.
#[cfg(feature = "automata")]
pub fn staging_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

    let boot_nodes = vec![];

    let initial_authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
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

//TODO: we need to update contextfree spec when we want to launch it officially
#[cfg(feature = "contextfree")]
fn contextfree_config_genesis(wasm_binary: &[u8]) -> contextfree::GenesisConfig {
    let mut endowed_accounts: Vec<(AccountId, u128)> = vec![
        //Chainbridge pallet account
        (
            AccountId::from_ss58check("a7Qbi6onJLQu6h37oLDRCoYbYcQ7B49Tz4gxNsPP5UT5bMy4B").unwrap(),
            300700000 * DOLLARS,
        ),
        //Team account
        (
            AccountId::from_ss58check("a7SvNLeY4LLRvUzmgEEBQwmdMExW5ZpPqBEvDDuB65nHF9hTk").unwrap(),
            150000000 * DOLLARS,
        ),
        //Advisor account
        (
            AccountId::from_ss58check("a7QJhiA62xQR4HsCtCyEgJDVQ6eLnsQFAMXpHeYUQtoict1Zy").unwrap(),
            50000000 * DOLLARS,
        ),
        //Eco & Dev community account
        (
            AccountId::from_ss58check("a7RFMEiomBwYPRfjDJKHKKwK1a2fdwsmB9w1bZgCyDXwMrCUa").unwrap(),
            220000000 * DOLLARS,
        ),
        //Protocol Reserve account
        (
            AccountId::from_ss58check("a7RSYT4AXr668NGpMC3vv7h5Jqb4HYNr4cqNt9M5H4nRrjz3b").unwrap(),
            279250000 * DOLLARS,
        ),
    ];

    let initial_authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )> = vec![
        (
            // a7PJf9QzaZpBxcZc1xwxCtXmexowt4i15zKDRmS3dr6cAmbjf
            hex!["34320a13ec8008a9b735f9acc991a1c54c399c377696f38661aa3c5fcb86f714"].into(),
            // a7SiPCDWkTpvmw7iWRyxpL9YKZnnHLFoYFvECBRzNFnkGsUzS
            hex!["cafeee6712174eb1aca3a417917e2ec2c48ac95dd2f1e86d362d801fbe2d2b65"].into(),
            // a7TjXAbsJ8H7hjjDr9CTiEhQqYfzbQEFkBMceDpjcHDDz6M2A
            hex!["f8193f2ed5a172117e517da0847f1056c0dc10b3df8695f78d91bc03c2a7a0f9"]
                .unchecked_into(),
            // a7SM7m8VQEudoEscP8antLbabujSmSRPM56JMDDgN9WPJxnCK
            hex!["bac6d3c69e0b13082363c93b6b440469a9066fb9dd720203d1c32e1b2fc9dc41"]
                .unchecked_into(),
            // a7SM7m8VQEudoEscP8antLbabujSmSRPM56JMDDgN9WPJxnCK
            hex!["bac6d3c69e0b13082363c93b6b440469a9066fb9dd720203d1c32e1b2fc9dc41"]
                .unchecked_into(),
            // a7SM7m8VQEudoEscP8antLbabujSmSRPM56JMDDgN9WPJxnCK
            hex!["bac6d3c69e0b13082363c93b6b440469a9066fb9dd720203d1c32e1b2fc9dc41"]
                .unchecked_into(),
        ),
        (
            // a7TXW1KoxRsDwZoGKY9jhFKFLvp1VB2mSony3mftp1P6F1txD
            hex!["eeee65b0e5fd2a934353083cf31e0737ebd161a27cf9312be07177ad4ee0fb7e"].into(),
            // a7RLS2NBBmYkT3PFdjxwRSVkVGJJaANJqCCHyouBwxcwyMenA
            hex!["8e04d37a3692c00496060e068dd0b3933a5e98ab6abca77488637972b4ab0421"].into(),
            // a7NJ1SZmHcBhDRboxU4XxaumwTB8Twmi3Qwnuqd2GbnrvY1PE
            hex!["0776e1c83d2bcfe140e5a50aaf96d19d68f22fda9b6e14c84d4a0d15e4e6ad6a"]
                .unchecked_into(),
            // a7PTUVStVQg5n5VRmTaB9LLSEEdLKJdNGqLVzX6zYTMuabyXQ
            hex!["3aeb61a30ffbe647e0dd6812ff0cf48a89239b693a824eeaa7c8bcc2ae81c11e"]
                .unchecked_into(),
            // a7PTUVStVQg5n5VRmTaB9LLSEEdLKJdNGqLVzX6zYTMuabyXQ
            hex!["3aeb61a30ffbe647e0dd6812ff0cf48a89239b693a824eeaa7c8bcc2ae81c11e"]
                .unchecked_into(),
            // a7PTUVStVQg5n5VRmTaB9LLSEEdLKJdNGqLVzX6zYTMuabyXQ
            hex!["3aeb61a30ffbe647e0dd6812ff0cf48a89239b693a824eeaa7c8bcc2ae81c11e"]
                .unchecked_into(),
        ),
        (
            // a7R5AVrzThhEZvJfuvDvvMz6Qmeurty6JVH6gWPvPoTgHMqM1
            hex!["825fe14fb483fa0339f45afa8e1784495803d4e6f1ce4c829765de41785b8457"].into(),
            // a7SqTMTAdzuutGopTNwQ5xAS81itXuuEW8rZ9X6WXRTyWCS5K
            hex!["d063a67ac0be52e416ca9786372ff92b86b5337381cf8750dc89ca657c14f02a"].into(),
            // a7TDArTsoC46Hs1Z62EatPEY8yGJZAfQrbLfqMSn8CLGc96L3
            hex!["e0f38186f2aac19f17cadf2c72f53e29e9aa8075af381db2bf6a8362e6428b65"]
                .unchecked_into(),
            // a7PEGQvTRYJQwme43nxaKxeJasfCB5SUZy8XPLbdpPmmniHrv
            hex!["30d8886dc8917f39950238ab3de264084fcf69097f47117f8f95e1c8eea3000e"]
                .unchecked_into(),
            // a7PEGQvTRYJQwme43nxaKxeJasfCB5SUZy8XPLbdpPmmniHrv
            hex!["30d8886dc8917f39950238ab3de264084fcf69097f47117f8f95e1c8eea3000e"]
                .unchecked_into(),
            // a7PEGQvTRYJQwme43nxaKxeJasfCB5SUZy8XPLbdpPmmniHrv
            hex!["30d8886dc8917f39950238ab3de264084fcf69097f47117f8f95e1c8eea3000e"]
                .unchecked_into(),
        ),
        (
            // a7QaE42qjki7qtMbc9pATT7UNE6j2UovqoRr2KL8NnSsb2KrG
            hex!["6c4dbabb5e92a381eace2fee7177095e7acb8a269062f2b55b6800d09e925841"].into(),
            // a7QAKxRxbSe5NerRFKo6AzpSpabvCnXA68qd5z4W1ZXHKg722
            hex!["5a13b660b947e70438b6d658db6fe3cf2ad2eb4ab0844f5c58c5056e8301da01"].into(),
            // a7Q82obVDqSzKj2FXw7cZuWfoRAYQ1yMSprCGsnwEVFwuW9Vp
            hex!["58537b4f730c4af54f221c4f8fbb5a2ba692f34f5738e86d8ed951dbaf149a5e"]
                .unchecked_into(),
            // a7RtFhm5h7ggMCFwMYAhWgF5dqPzcH18AywnjHVXs9uQ7MhUy
            hex!["a649efaa23870cc28b6ab624bc46e169c509362775442b02553700256f858135"]
                .unchecked_into(),
            // a7RtFhm5h7ggMCFwMYAhWgF5dqPzcH18AywnjHVXs9uQ7MhUy
            hex!["a649efaa23870cc28b6ab624bc46e169c509362775442b02553700256f858135"]
                .unchecked_into(),
            // a7RtFhm5h7ggMCFwMYAhWgF5dqPzcH18AywnjHVXs9uQ7MhUy
            hex!["a649efaa23870cc28b6ab624bc46e169c509362775442b02553700256f858135"]
                .unchecked_into(),
        ),
    ];

    initial_authorities.iter().for_each(|x| {
        endowed_accounts.push((x.0.clone(), 9000 * DOLLARS));
        endowed_accounts.push((x.1.clone(), 1000 * DOLLARS));
    });

    let root_key: AccountId =
        AccountId::from_ss58check("a7PywYxBDEBYTAfYPFGWEghzCFcTmp6fMvDR51sMMf2sotgAX").unwrap();
    endowed_accounts.push((root_key.clone(), 10000 * DOLLARS));

    contextfree::GenesisConfig {
        system: contextfree::SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: contextfree::BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k.0, k.1))
                .collect(),
        },
        indices: contextfree::IndicesConfig { indices: vec![] },
        babe: contextfree::BabeConfig {
            authorities: vec![],
            epoch_config: Some(contextfree::BABE_GENESIS_EPOCH_CONFIG),
        },
        grandpa: contextfree::GrandpaConfig {
            authorities: vec![],
        },
        staking: contextfree::StakingConfig {
            validator_count: 4,
            minimum_validator_count: 2,
            stakers: vec![],
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: sp_runtime::Perbill::from_percent(10),
            force_era: pallet_staking::Forcing::ForceNone,
            ..Default::default()
        },
        session: contextfree::SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(), // stash
                        x.0.clone(), // stash
                        get_contextfree_session_keys(
                            x.2.clone(), // grandpa
                            x.3.clone(), // babe
                            x.4.clone(),
                            x.5.clone(),
                        ),
                    )
                })
                .collect::<Vec<_>>(),
        },
        im_online: contextfree::ImOnlineConfig { keys: vec![] },
        authority_discovery: contextfree::AuthorityDiscoveryConfig { keys: vec![] },
        democracy: contextfree::DemocracyConfig::default(),
        council: contextfree::CouncilConfig {
            members: vec![],
            phantom: Default::default(),
        },
        technical_committee: contextfree::TechnicalCommitteeConfig {
            members: vec![],
            phantom: Default::default(),
        },
        phragmen_election: contextfree::PhragmenElectionConfig::default(),
        technical_membership: contextfree::TechnicalMembershipConfig::default(),
        treasury: contextfree::TreasuryConfig::default(),
        evm: contextfree::EVMConfig::default(),
        ethereum: contextfree::EthereumConfig::default(),
        sudo: contextfree::SudoConfig {
            // Assign network admin rights.
            key: root_key,
        },
        vesting: contextfree::VestingConfig::default(),
    }
}

/// Configure initial storage state for FRAME modules.
#[cfg(feature = "automata")]
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )>,
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
            PalletId(*b"ata/brdg").into_account(), //5EYCAe5fjB53Kn9DfqH5G7M589vF4dQRbgAwwQs1fW7Wj1mY
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
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, ENDOWMENT))
                .collect(),
        },
        indices: IndicesConfig { indices: vec![] },
        session: SessionConfig {
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
                            x.5.clone(),
                        ),
                    )
                })
                .collect::<Vec<_>>(),
        },
        staking: StakingConfig {
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
        },
        babe: BabeConfig {
            authorities: vec![],
            epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG),
        },
        grandpa: GrandpaConfig {
            authorities: vec![],
        },
        authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
        im_online: ImOnlineConfig { keys: vec![] },
        // pallet_grandpa: Some(GrandpaConfig {
        //     authorities: initial_authorities
        //         .iter()
        //         .map(|x| (x.2.clone(), 1))
        //         .collect(),
        // }),
        sudo: SudoConfig {
            // Assign network admin rights.
            key: root_key,
        },
        evm: EVMConfig {
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
        },
        ethereum: EthereumConfig::default(),
    }
}
