#[cfg(all(feature = "automata", feature = "contextfree"))]
compile_error!("Feature 1 and 2 are mutually exclusive and cannot be enabled together");
#[cfg(all(feature = "automata", feature = "finitestate"))]
compile_error!("Feature 1 and 2 are mutually exclusive and cannot be enabled together");
#[cfg(all(feature = "finitestate", feature = "contextfree"))]
compile_error!("Feature 1 and 2 are mutually exclusive and cannot be enabled together");

use automata_primitives::Block;
pub use automata_primitives::{AccountId, Balance, BlockNumber, Signature};

#[cfg(feature = "automata")]
use automata_runtime as automata;
#[cfg(feature = "automata")]
use automata_runtime::{constants::currency::*, GenesisConfig};
#[cfg(feature = "contextfree")]
use contextfree_runtime as contextfree;
#[cfg(feature = "contextfree")]
use contextfree_runtime::{constants::currency::*, GenesisConfig};
#[cfg(feature = "finitestate")]
use finitestate::{constants::currency::*, GenesisConfig, StakerStatus};
#[cfg(feature = "finitestate")]
use finitestate_runtime as finitestate;
#[cfg(feature = "finitestate")]
use frame_support::PalletId;
#[cfg(feature = "finitestate")]
use sp_core::{H160, U256};
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
    sr25519, Pair, Public
};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};

#[cfg(feature = "automata")]
pub type AutomataChainSpec = sc_service::GenericChainSpec<automata::GenesisConfig, Extensions>;

#[cfg(feature = "contextfree")]
pub type ContextFreeChainSpec =
    sc_service::GenericChainSpec<contextfree::GenesisConfig, Extensions>;

#[cfg(feature = "finitestate")]
pub type FiniteStateChainSpec =
    sc_service::GenericChainSpec<finitestate::GenesisConfig, Extensions>;

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

#[cfg(feature = "finitestate")]
fn get_properties() -> Option<Properties> {
    let mut properties = Properties::new();
    properties.insert("tokenSymbol".into(), "FST".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("ss58Format".into(), 13107.into());
    Some(properties)
}

#[cfg(feature = "automata")]
pub fn automata_chain_spec() -> Result<AutomataChainSpec, String> {
    AutomataChainSpec::from_json_bytes(&include_bytes!("../../assets/chain_spec_automata.json")[..])
}

#[cfg(feature = "contextfree")]
pub fn contextfree_chain_spec() -> Result<ContextFreeChainSpec, String> {
    ContextFreeChainSpec::from_json_bytes(
        &include_bytes!("../../assets/chain_spec_contextfree.json")[..],
    )
}

#[cfg(feature = "finitestate")]
pub fn finitestate_chain_spec() -> Result<FiniteStateChainSpec, String> {
    FiniteStateChainSpec::from_json_bytes(
        &include_bytes!("../../assets/chain_spec_finitestate.json")[..],
    )
}

#[cfg(feature = "automata")]
fn get_session_keys(
    grandpa: GrandpaId,
    babe: BabeId,
    im_online: ImOnlineId,
    authority_discovery: AuthorityDiscoveryId,
) -> automata::opaque::SessionKeys {
    automata::opaque::SessionKeys {
        babe,
        grandpa,
        im_online,
        authority_discovery,
    }
}

#[cfg(feature = "contextfree")]
fn get_session_keys(
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

#[cfg(feature = "finitestate")]
fn get_session_keys(
    grandpa: GrandpaId,
    babe: BabeId,
    im_online: ImOnlineId,
    authority_discovery: AuthorityDiscoveryId,
) -> finitestate::opaque::SessionKeys {
    finitestate::opaque::SessionKeys {
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

#[cfg(feature = "finitestate")]
pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary =
        finitestate::WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

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

#[cfg(feature = "finitestate")]
pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary =
        finitestate::WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

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

#[cfg(feature = "automata")]
pub fn automata_testnet_config() -> Result<AutomataChainSpec, String> {
    let wasm_binary = automata::WASM_BINARY.ok_or("Automata testnet awsm not available")?;
    let boot_nodes = vec![];

    Ok(AutomataChainSpec::from_genesis(
        "Automata Network",
        "automata_network",
        ChainType::Live,
        move || automata_config_genesis(wasm_binary),
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

#[cfg(feature = "finitestate")]
pub fn finitestate_testnet_config() -> Result<FiniteStateChainSpec, String> {
    let wasm_binary = finitestate::WASM_BINARY.ok_or("Finitestate testnet awsm not available")?;
    let boot_nodes = vec![];

    Ok(FiniteStateChainSpec::from_genesis(
        "FiniteState Network",
        "finitestate_network",
        ChainType::Live,
        move || finitestate_config_genesis(wasm_binary),
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

#[cfg(feature = "automata")]
fn automata_genesis_accounts() -> (
    Vec<(AccountId, u128)>,
    Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )>,
    Vec<(AccountId, u64, u64, u64, u64, u128, u128, bool)>,
    AccountId,
) {
    let endowed_accounts: Vec<(AccountId, u128)> = vec![
        //Chainbridge pallet account
        (
            AccountId::from_ss58check("aA6rX5bQBiGWiud3KnbAqzsn24Vy7Y6U3A5uvUqkCyY8kxeFf").unwrap(),
            246000000 * DOLLARS,
        ),
        //Team account
        (
            AccountId::from_ss58check("aA8zrbUXa5SkUwe2EBrQJC7aeuBfX3M4vFmwXvwGu6aF4A69o").unwrap(),
            150000000 * DOLLARS,
        ),
        //Advisor account
        (
            AccountId::from_ss58check("aA8soqLqZj98EQhktfywvipkcZLDDdhMPUmnfLGEspub8FjoV").unwrap(),
            50000000 * DOLLARS,
        ),
        //Eco & Dev community account
        (
            AccountId::from_ss58check("aA8ue1WwLD6k4rvgaRCBLwpT25JCPgMqzV1NnZJ1RgrL42WyN").unwrap(),
            259930000 * DOLLARS,
        ),
        //Protocol Reserve account
        (
            AccountId::from_ss58check("aA8RW3kstyvQ2P7zMh79HCtWBSCwxipB18N1Z1pB8XQa3gFpP").unwrap(),
            294000000 * DOLLARS,
        ),
    ];

    let vesting_plans: Vec<(AccountId, u64, u64, u64, u64, u128, u128, bool)> = vec![
        (
            // Team
            AccountId::from_ss58check("aA8zrbUXa5SkUwe2EBrQJC7aeuBfX3M4vFmwXvwGu6aF4A69o").unwrap(),
            1623045420000,       //start time: 7th June 2021
            15552000000,         //cliff duration: 6 months
            155520000000,        //total duration: 60 months
            7776000000,          //interval: 3 months
            0,                   //initial amount
            150000000 * DOLLARS, //total amount
            true,                // vesting during cliff
        ),
        (
            // Advisor
            AccountId::from_ss58check("aA8soqLqZj98EQhktfywvipkcZLDDdhMPUmnfLGEspub8FjoV").unwrap(),
            1623045420000,      //start time: 7th June 2021
            15552000000,        //cliff duration: 6 months
            155520000000,       //total duration: 60 months
            7776000000,         //interval: 3 months
            0,                  //initial amount
            50000000 * DOLLARS, //total amount
            true,               // vesting during cliff
        ),
        (
            // Eco & Dev
            AccountId::from_ss58check("aA8ue1WwLD6k4rvgaRCBLwpT25JCPgMqzV1NnZJ1RgrL42WyN").unwrap(),
            1630821420000,       //start time (1st release + 1 interval): 5th Sep 2021
            0,                   //cliff duration
            85536000000,         //total duration (total duration - 1 interval): 33 months
            7776000000,          //interval: 3 months
            39930000 * DOLLARS,  //initial amount (1st release + 2nd release - validator - sudo)
            259930000 * DOLLARS, //total amount (total - validator - sudo)
            false,               // vesting during cliff
        ),
        (
            // Protocol
            AccountId::from_ss58check("aA8RW3kstyvQ2P7zMh79HCtWBSCwxipB18N1Z1pB8XQa3gFpP").unwrap(),
            1623045420000,       //start time: 7th June 2021
            0,                   //cliff duration
            155520000000,        //total duration: 60 months
            7776000000,          //interval: 3 months
            0,                   //initial amount
            294000000 * DOLLARS, //total amount
            false,               // vesting during cliff
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
            // aA8BbdCuXJMhsYRpLtrWTc4D6pKm6LfoanMNuT31P8PaLn8ZY
            hex!["a838969fb3bcc567ac56ef19aad14fd59a4bcee4c547247ed03298a13ab1125c"].into(),
            // aA9Gtmffr3KQrRmLvkym4wzABmDRsLZ8x9exaoVzY5oUnUZ6k
            hex!["d87ec45d6662f6974eed63602d7e149bcd67821bf80d25e51e717d520a4b4427"].into(),
            // aA9LXQbR9tkbqPJhkV4p1EhuJ3fKfa7b4CzkFMAeBbTh4ksPf
            hex!["db43d551a4b807e2158685714a2e2169ab84a57ec667cd5533af46b609039bd2"]
                .unchecked_into(),
            // aA5VGJ5ggnoT4ksfjwLctBSXFZDK96VFwwmAHQYLzNSq4D2aw
            hex!["30fd3c511e73f801c02d67ebdb975d624d4c568b1f5f413762b2454f709bd20b"]
                .unchecked_into(),
            // aA5VGJ5ggnoT4ksfjwLctBSXFZDK96VFwwmAHQYLzNSq4D2aw
            hex!["30fd3c511e73f801c02d67ebdb975d624d4c568b1f5f413762b2454f709bd20b"]
                .unchecked_into(),
            // aA5VGJ5ggnoT4ksfjwLctBSXFZDK96VFwwmAHQYLzNSq4D2aw
            hex!["30fd3c511e73f801c02d67ebdb975d624d4c568b1f5f413762b2454f709bd20b"]
                .unchecked_into(),
        ),
        (
            // aA9jHYZRkv4someHCmqRw8TwZPrNz89LiTtX5wDH13Bu4tP9H
            hex!["ec9fcfcbbdd4b3ba2afbe39ce5cf508885203f5ef313ae42e1eff40d4f100526"].into(),
            // aA5ZPiyupm1XP8NnGFKAw6ErprodgAFtGexbXjV6W21kZn5na
            hex!["342339fed5b97ff4033bdb23ff0fcf506a16ddece2c53776671c79b2688cf618"].into(),
            // aA5Z6P8RpDAiTfKX1zHNa7hf98uvqrARAW8wWpUgXdLnHqD8w
            hex!["33e8d8e05805353c28ae0cd2336db82ba1949ee4996486bb2308bba1ec562c3d"]
                .unchecked_into(),
            // aA4qHvrPWeDCNRBL3N3kjwxmXg7tLn9quJKHfsDqzRkEHuhyY
            hex!["140769243cb70a9c3fe2c73aeada402785ca6ac2f69610071da9e321d5c1af61"]
                .unchecked_into(),
            // aA4qHvrPWeDCNRBL3N3kjwxmXg7tLn9quJKHfsDqzRkEHuhyY
            hex!["140769243cb70a9c3fe2c73aeada402785ca6ac2f69610071da9e321d5c1af61"]
                .unchecked_into(),
            // aA4qHvrPWeDCNRBL3N3kjwxmXg7tLn9quJKHfsDqzRkEHuhyY
            hex!["140769243cb70a9c3fe2c73aeada402785ca6ac2f69610071da9e321d5c1af61"]
                .unchecked_into(),
        ),
        (
            // aA9gJP6NP8N7usGvLdYqMQT4MNLpPoohrSo2JsjUMVD44fQ2y
            hex!["ea58e4717103834b82d97bff600a8ebd345e384272a3782577eb40fdaed7aa4b"].into(),
            // aA8YLLC6iGMJRCBufaehgsCNCfVVrS16ApR7Ejkoi7cRfNTxW
            hex!["b8094287b1c1af37437c477fad4f2c6a350a98028aa9727f7c442efdee3a1c58"].into(),
            // aA93iHshYQLvup8mNJKNGjcsvQ4GsPP8ncrK8coid2NMPys6m
            hex!["ce714cfbf906b51d720f39ae184f49b5090669fd21d4b5c1f5548d0a3878abd5"]
                .unchecked_into(),
            // aA4fb93xenFEZLRND2vwEP6B6iBvNRU7YsWGBxbsjq1fSKzsL
            hex!["0ca0e44e72ad0214f7cb493b0625b7f799bca820d0c9491cc3640b0c61ff1a77"]
                .unchecked_into(),
            // aA4fb93xenFEZLRND2vwEP6B6iBvNRU7YsWGBxbsjq1fSKzsL
            hex!["0ca0e44e72ad0214f7cb493b0625b7f799bca820d0c9491cc3640b0c61ff1a77"]
                .unchecked_into(),
            // aA4fb93xenFEZLRND2vwEP6B6iBvNRU7YsWGBxbsjq1fSKzsL
            hex!["0ca0e44e72ad0214f7cb493b0625b7f799bca820d0c9491cc3640b0c61ff1a77"]
                .unchecked_into(),
        ),
        (
            // aA4ZkXXUgsC6LpNtmFZRfX8BJUyM3WxNxkvfWc3UDdi5Vgwfk
            hex!["082d053fdba5b21d4d8cb2323ab47284952b6e20009a900cce3e9c77fe8cfa67"].into(),
            // aA815THPp9A42z9DamHKPAU13fJZd6Baeh19SueZBK2Uo3CRe
            hex!["a03291c5b093377028c0cbcb465bcd4c77b9bb9820485ce1be8a1b8ad2f18453"].into(),
            // aA5FiZCabBdVh5QfDD88hnvMNBAYZsua5g3MUTxVYoLYTAy2N
            hex!["26a83384ff1712afeac68bf7a2b5703634d18931e14e695fa8b68779317c4986"]
                .unchecked_into(),
            // aA8gJWUPsyorgGn4o3ThJxQfAd4s2iniEmfbNxHub5MphoyJA
            hex!["be1d1654257e5101b307bbd42115b304442635d26be45337bbadd1f357f6b76b"]
                .unchecked_into(),
            // aA8gJWUPsyorgGn4o3ThJxQfAd4s2iniEmfbNxHub5MphoyJA
            hex!["be1d1654257e5101b307bbd42115b304442635d26be45337bbadd1f357f6b76b"]
                .unchecked_into(),
            // aA8gJWUPsyorgGn4o3ThJxQfAd4s2iniEmfbNxHub5MphoyJA
            hex!["be1d1654257e5101b307bbd42115b304442635d26be45337bbadd1f357f6b76b"]
                .unchecked_into(),
        ),
        (
            // aA5aShajm2P7oppAKLoKzyWZhfkjuTCwKiSAnZAqrWxVbuzf3
            hex!["34f07d57c141f552a615c2a5a174b1fa41339dc285be289ba3c279af07e2650e"].into(),
            // aA6RfD7YYn36DcV2jMxwH7kRoB2C27DX6GwpUjS1EkVH43vyv
            hex!["5a799fbf4c1d247730a004409dbca914fda837df166db39698e74b6bfa87ca47"].into(),
            // aA5YrPy1CyEm97U6EHiMFu7tizeTYzgJjWnJu95pKeDeGQmAa
            hex!["33b9c4803204d391074adf6f0b25ef74798d73a13f179dcbbf7e0fc2ec704c78"]
                .unchecked_into(),
            // aA9MuBQTNaExeM8nPZ9o9A3ncE5Vw7JKx8cKtK2AkAu4UvdAp
            hex!["dc505f2e2bdfdbe8f847d8c11e731545428de8732b923706c69b9ac52b31d663"]
                .unchecked_into(),
            // aA9MuBQTNaExeM8nPZ9o9A3ncE5Vw7JKx8cKtK2AkAu4UvdAp
            hex!["dc505f2e2bdfdbe8f847d8c11e731545428de8732b923706c69b9ac52b31d663"]
                .unchecked_into(),
            // aA9MuBQTNaExeM8nPZ9o9A3ncE5Vw7JKx8cKtK2AkAu4UvdAp
            hex!["dc505f2e2bdfdbe8f847d8c11e731545428de8732b923706c69b9ac52b31d663"]
                .unchecked_into(),
        ),
        (
            // aA6gwkfiZpeMwRDH4fvFYFfYiG8ScfLSf2V97Uud5BbrFnj5x
            hex!["6621ff4f46d7ceeeb5a7573168a40ac71d8128983a3387ff5098a804159a4c15"].into(),
            // aA7WsJ2F8btELxWjmbtTC13CTiMbDDq2SFj4A72XuFWTXD6Xw
            hex!["8aaec3d5e93e5ddacb9d0aa18b08167f764426b33f9cf2ce34d7585326ac2439"].into(),
            // aA8vKCbFAFUVhhgN5RqcFvkqqP9XS7XLs5J8Y5AFYRdhtf8mA
            hex!["c8ccd9f05042cce75050d7e93aadc43d79dfe75b5c9bd058823759a6118527a9"]
                .unchecked_into(),
            // aA52HREYNrwWY7sJJXjFNT6PxQznnaZVeMPauhcQDAWmXLZkx
            hex!["1c696421745eda86fc80b68ef6e382f5400aa8475c3f950e0363454b992c2051"]
                .unchecked_into(),
            // aA52HREYNrwWY7sJJXjFNT6PxQznnaZVeMPauhcQDAWmXLZkx
            hex!["1c696421745eda86fc80b68ef6e382f5400aa8475c3f950e0363454b992c2051"]
                .unchecked_into(),
            // aA52HREYNrwWY7sJJXjFNT6PxQznnaZVeMPauhcQDAWmXLZkx
            hex!["1c696421745eda86fc80b68ef6e382f5400aa8475c3f950e0363454b992c2051"]
                .unchecked_into(),
        ),
    ];

    let root_key: AccountId =
        AccountId::from_ss58check("aA8rnWtLvNhtxVR5umiw4eM8uhCbGJTcJYBEVRFSv1VKZ7JnW").unwrap();

    (
        endowed_accounts,
        initial_authorities,
        vesting_plans,
        root_key,
    )
}

#[cfg(feature = "contextfree")]
fn contextfree_genesis_accounts() -> (
    Vec<(AccountId, u128)>,
    Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )>,
    AccountId,
) {
    let endowed_accounts: Vec<(AccountId, u128)> = vec![
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

    let root_key: AccountId =
        AccountId::from_ss58check("a7PywYxBDEBYTAfYPFGWEghzCFcTmp6fMvDR51sMMf2sotgAX").unwrap();

    (endowed_accounts, initial_authorities, root_key)
}

#[cfg(feature = "finitestate")]
fn finitestate_genesis_accounts() -> (
    Vec<(AccountId, u128)>,
    Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )>,
    AccountId,
) {
    let endowed_accounts: Vec<(AccountId, u128)> = vec![
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
            279270000 * DOLLARS,
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
    ];

    let root_key: AccountId =
        AccountId::from_ss58check("a7PywYxBDEBYTAfYPFGWEghzCFcTmp6fMvDR51sMMf2sotgAX").unwrap();

    (endowed_accounts, initial_authorities, root_key)
}

#[cfg(feature = "automata")]
fn automata_config_genesis(wasm_binary: &[u8]) -> automata::GenesisConfig {
    let (mut endowed_accounts, initial_authorities, vesting_plans, root_key) =
        automata_genesis_accounts();

    endowed_accounts.push((root_key.clone(), 10000 * DOLLARS));

    initial_authorities.iter().for_each(|x| {
        endowed_accounts.push((x.0.clone(), 9000 * DOLLARS));
        endowed_accounts.push((x.1.clone(), 1000 * DOLLARS));
    });

    automata::GenesisConfig {
        system: automata::SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: automata::BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k.0, k.1))
                .collect(),
        },
        indices: automata::IndicesConfig { indices: vec![] },
        babe: automata::BabeConfig {
            authorities: vec![],
            epoch_config: Some(automata::BABE_GENESIS_EPOCH_CONFIG),
        },
        grandpa: automata::GrandpaConfig {
            authorities: vec![],
        },
        staking: automata::StakingConfig {
            validator_count: 6,
            minimum_validator_count: 3,
            stakers: vec![],
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: sp_runtime::Perbill::from_percent(10),
            force_era: pallet_staking::Forcing::ForceNone,
            ..Default::default()
        },
        session: automata::SessionConfig {
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
        im_online: automata::ImOnlineConfig { keys: vec![] },
        authority_discovery: automata::AuthorityDiscoveryConfig { keys: vec![] },
        democracy: automata::DemocracyConfig::default(),
        council: automata::CouncilConfig {
            members: vec![],
            phantom: Default::default(),
        },
        technical_committee: automata::TechnicalCommitteeConfig {
            members: vec![],
            phantom: Default::default(),
        },
        phragmen_election: automata::PhragmenElectionConfig::default(),
        technical_membership: automata::TechnicalMembershipConfig::default(),
        treasury: automata::TreasuryConfig::default(),
        evm: automata::EVMConfig::default(),
        ethereum: automata::EthereumConfig::default(),
        sudo: automata::SudoConfig {
            // Assign network admin rights.
            key: root_key,
        },
        vesting: automata::VestingConfig {
            vesting: vesting_plans,
        },
    }
}

//TODO: we need to update contextfree spec when we want to launch it officially
#[cfg(feature = "contextfree")]
fn contextfree_config_genesis(wasm_binary: &[u8]) -> contextfree::GenesisConfig {
    let (mut endowed_accounts, initial_authorities, root_key) = contextfree_genesis_accounts();

    endowed_accounts.push((root_key.clone(), 10000 * DOLLARS));

    initial_authorities.iter().for_each(|x| {
        endowed_accounts.push((x.0.clone(), 9000 * DOLLARS));
        endowed_accounts.push((x.1.clone(), 1000 * DOLLARS));
    });

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

#[cfg(feature = "finitestate")]
fn finitestate_config_genesis(wasm_binary: &[u8]) -> finitestate::GenesisConfig {
    let (mut endowed_accounts, initial_authorities, root_key) = finitestate_genesis_accounts();
    endowed_accounts.push((root_key.clone(), 10000 * DOLLARS));

    initial_authorities.iter().for_each(|x| {
        endowed_accounts.push((x.0.clone(), 9000 * DOLLARS));
        endowed_accounts.push((x.1.clone(), 1000 * DOLLARS));
    });

    finitestate::GenesisConfig {
        system: finitestate::SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: finitestate::BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k.0, k.1))
                .collect(),
        },
        indices: finitestate::IndicesConfig { indices: vec![] },
        babe: finitestate::BabeConfig {
            authorities: vec![],
            epoch_config: Some(finitestate::BABE_GENESIS_EPOCH_CONFIG),
        },
        grandpa: finitestate::GrandpaConfig {
            authorities: vec![],
        },
        staking: finitestate::StakingConfig {
            validator_count: 2,
            minimum_validator_count: 2,
            stakers: vec![],
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: sp_runtime::Perbill::from_percent(10),
            force_era: pallet_staking::Forcing::ForceNone,
            ..Default::default()
        },
        session: finitestate::SessionConfig {
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
        im_online: finitestate::ImOnlineConfig { keys: vec![] },
        authority_discovery: finitestate::AuthorityDiscoveryConfig { keys: vec![] },
        democracy: finitestate::DemocracyConfig::default(),
        council: finitestate::CouncilConfig {
            members: vec![],
            phantom: Default::default(),
        },
        technical_committee: finitestate::TechnicalCommitteeConfig {
            members: vec![],
            phantom: Default::default(),
        },
        phragmen_election: finitestate::PhragmenElectionConfig::default(),
        technical_membership: finitestate::TechnicalMembershipConfig::default(),
        treasury: finitestate::TreasuryConfig::default(),
        evm: finitestate::EVMConfig::default(),
        ethereum: finitestate::EthereumConfig::default(),
        sudo: finitestate::SudoConfig {
            // Assign network admin rights.
            key: root_key,
        },
        vesting: finitestate::VestingConfig::default(),
    }
}

/// Configure initial storage state for FRAME modules.
#[cfg(feature = "finitestate")]
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
        system: finitestate::SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: finitestate::BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, ENDOWMENT))
                .collect(),
        },
        indices: finitestate::IndicesConfig { indices: vec![] },
        session: finitestate::SessionConfig {
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
        staking: finitestate::StakingConfig {
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
        babe: finitestate::BabeConfig {
            authorities: vec![],
            epoch_config: Some(finitestate::BABE_GENESIS_EPOCH_CONFIG),
        },
        grandpa: finitestate::GrandpaConfig {
            authorities: vec![],
        },
        authority_discovery: finitestate::AuthorityDiscoveryConfig { keys: vec![] },
        democracy: finitestate::DemocracyConfig::default(),
        council: finitestate::CouncilConfig {
            members: vec![],
            phantom: Default::default(),
        },
        technical_committee: finitestate::TechnicalCommitteeConfig {
            members: vec![],
            phantom: Default::default(),
        },
        phragmen_election: finitestate::PhragmenElectionConfig::default(),
        technical_membership: finitestate::TechnicalMembershipConfig::default(),
        treasury: finitestate::TreasuryConfig::default(),
        im_online: finitestate::ImOnlineConfig { keys: vec![] },
        // pallet_grandpa: Some(GrandpaConfig {
        //     authorities: initial_authorities
        //         .iter()
        //         .map(|x| (x.2.clone(), 1))
        //         .collect(),
        // }),
        sudo: finitestate::SudoConfig {
            // Assign network admin rights.
            key: root_key,
        },
        evm: finitestate::EVMConfig {
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
        ethereum: finitestate::EthereumConfig::default(),
        vesting: finitestate::VestingConfig::default(),
    }
}
