use authority_discovery_primitives::AuthorityId as AuthorityDiscoveryId;
use babe_primitives::AuthorityId as BabeId;
use grandpa::AuthorityId as GrandpaId;
use hex_literal::hex;
use im_online::sr25519::AuthorityId as ImOnlineId;
use node_primitives::{AccountId, Balance, Signature};
use node_runtime::constants::currency::*;
use node_runtime::Block;
use node_runtime::{
    AuthorityDiscoveryConfig, BabeConfig, BalancesConfig, CouncilConfig, DemocracyConfig,
    GenesisConfig, GrandpaConfig, ImOnlineConfig, IndicesConfig, SessionConfig, SessionKeys,
    StakerStatus, StakingConfig, SudoConfig, SystemConfig, TechnicalCommitteeConfig, WASM_BINARY,
};
use primitives::crypto::UncheckedInto;
use primitives::{sr25519, Pair, Public};
use sc_chain_spec::ChainSpecExtension;
use serde::{Deserialize, Serialize};
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    Perbill,
};

use telemetry::TelemetryEndpoints;

const STAGING_TELEMETRY_URL: &str = "ws://telemetry.mvsdna.com:8000/submit";

// TODO: Remove if not needed
const DNA_PROTOCOL_ID: &str = "dna"; // we dont need this
const DNA_PROPERTIES: &str = r#"
    {
        "tokenDecimals": 4,
        "tokenSymbol": "DNA"
    }"#;

type AccountPublic = <Signature as Verify>::Signer;

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    /// Block numbers with known hashes.
    pub fork_blocks: sc_client::ForkBlocks<Block>,
    /// Known bad block hashes.
    pub bad_blocks: sc_client::BadBlocks<Block>,
}

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;
/// The chain specification option. This is expected to come in from the CLI and
/// is little more than one of a number of alternatives which can easily be converted
/// from a string (`--chain=...`) into a `ChainSpec`.
#[derive(Clone, Debug)]
enum ChainOpt {
    /// Whatever the current runtime is, with just Alice as an auth.
    Development,
    /// Whatever the current runtime is, with simple Alice/Bob auths.
    LocalTestnet,
    /// DNA public testnet.
    DNATestnet,
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
pub fn get_authority_keys_from_seed(
    seed: &str,
) -> (
    AccountId,
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
) {
    (
        get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<BabeId>(seed),
        get_from_seed::<ImOnlineId>(seed),
        get_from_seed::<AuthorityDiscoveryId>(seed),
    )
}

fn session_keys(
    grandpa: GrandpaId,
    babe: BabeId,
    im_online: ImOnlineId,
    authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
    SessionKeys {
        grandpa,
        babe,
        im_online,
        authority_discovery,
    }
}

/// Helper function to create GenesisConfig for testing
pub fn testnet_genesis(
    initial_authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )>,
    endowed_accounts: Option<Vec<AccountId>>,
) -> GenesisConfig {
    let endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
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

    const ENDOWMENT: Balance = 100 * DNA;
    const STASH: Balance = 1_000 * DNA;

    GenesisConfig {
        system: Some(SystemConfig {
            code: WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        indices: Some(IndicesConfig { indices: vec![] }),
        balances: Some(BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, ENDOWMENT))
                .chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
                .collect(),
        }),
        session: Some(SessionConfig {
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
        staking: Some(StakingConfig {
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
        democracy: Some(DemocracyConfig::default()),
        collective_Instance1: Some(CouncilConfig {
            members: vec![],
            phantom: Default::default(),
        }),
        collective_Instance2: Some(TechnicalCommitteeConfig {
            members: vec![],
            phantom: Default::default(),
        }),
        membership_Instance1: Some(Default::default()),
        sudo: Some(SudoConfig {
            key: endowed_accounts[0].clone(),
        }),
        babe: Some(BabeConfig {
            authorities: vec![],
        }),
        grandpa: Some(GrandpaConfig {
            authorities: vec![],
        }),
        im_online: Some(ImOnlineConfig { keys: vec![] }),
        authority_discovery: Some(AuthorityDiscoveryConfig { keys: vec![] }),
    }
}

// DNA Testnet config for testing and development
fn dna_config_genesis() -> GenesisConfig {
    let initial_authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )> = vec![
        (
            hex!["702078e8037ac21bd9dd872cf4dd87ad9c63172d24f66139ad5891412b95674f"].into(),
            hex!["702078e8037ac21bd9dd872cf4dd87ad9c63172d24f66139ad5891412b95674f"].into(),
            hex!["833c7b8896ef66f70c0ceb079e9c13772d1cdea271248ebfe0587a7fbc2ffef9"]
                .unchecked_into(),
            hex!["8e74d42e90c1d6fa1f6bf332a0ae79e896d84fbb189160bba6e79cdbccc0224f"]
                .unchecked_into(),
            hex!["9cfa107b702bed0e9a608dffdf9fc5b96a6a059677d393d244e8f53e79ad9e3f"]
                .unchecked_into(),
            hex!["1c9d3d4c3079a047f83f2ddf671d76c415381cedb0b2806752b66b7dbf6d3c76"]
                .unchecked_into(),
        ),
        (
            hex!["1c48ba97c86cbb2f69f3cb5948c002b3f01b57ad19add22f66f2d6ea6dd9f749"].into(),
            hex!["1c48ba97c86cbb2f69f3cb5948c002b3f01b57ad19add22f66f2d6ea6dd9f749"].into(),
            hex!["46d4ba514507111c3a6cf55c835128b0fb98d6503bd7ea71b4cd5eed3caab503"]
                .unchecked_into(),
            hex!["002051bfa1605e898f5f2bb114b3350d637a19915589f222728bdbcf21718535"]
                .unchecked_into(),
            hex!["e6748efd1cd8ddef64da4d39a702231a9fbcf0be45fc83b1d4abac5b42f3966b"]
                .unchecked_into(),
            hex!["3a9b58228202c62d075f495d5d71949492aa1fd79f3ea4a983cde443429fe124"]
                .unchecked_into(),
        ),
        (
            hex!["44d3c14dd109596426ac510f047e676a53d3d875653e7f43ead7ac4338ae931d"].into(),
            hex!["44d3c14dd109596426ac510f047e676a53d3d875653e7f43ead7ac4338ae931d"].into(),
            hex!["416f54bfd95ad9163b2702359249d38a92c9e9ffa7f93b169952fda7bd852365"]
                .unchecked_into(),
            hex!["def92d98a85bdf93fa2020aba5f223b82134c3024a0b7da5f23a424ecf224b57"]
                .unchecked_into(),
            hex!["42c1874ec13edc69e435845aaca2512957c06a5c916ac4e580e63f2273265b42"]
                .unchecked_into(),
            hex!["163cebbe85b0723ada2460de564dca7a97bdccbd06c57520133f292417db0370"]
                .unchecked_into(),
        ),
    ];

    let endowed_accounts: Vec<AccountId> = vec![
        // 5EbisDGXTdMScRusn6vBZ9B5QASm5LMRCVkERjRnUpz6bpi4
        hex!["702078e8037ac21bd9dd872cf4dd87ad9c63172d24f66139ad5891412b95674f"].into(),
        // 5ChnoXHLockcTcB1v6JMTa7H6cJZ7zFwPqBDHT6K8df4fc2N
        hex!["1c48ba97c86cbb2f69f3cb5948c002b3f01b57ad19add22f66f2d6ea6dd9f749"].into(),
        // 5Dcx1ysWu6g9VBgawLN6gbczPeGGX7PxfrdscCuZRCkWAZbr
        hex!["44d3c14dd109596426ac510f047e676a53d3d875653e7f43ead7ac4338ae931d"].into(),
    ];

    //    info!( "Hello --------------------------------------------------");

    testnet_genesis(initial_authorities, Some(endowed_accounts))
}

// pub fn dna_testnet_config() -> ChainSpec {
//     ChainSpec::from_json_bytes(&include_bytes!("../res/spec.dna.json")[..]).unwrap()
// }

/// testnet config.
pub fn dna_testnet_config() -> ChainSpec {
    let boot_nodes = vec![
        // validator-01
        "/ip4/127.0.0.1/tcp/3033/p2p/Qmece3bstSKgRomhPcAWswQMUFT3GRL5XpCuy8bFDggrwV".into(),
        "/ip4/127.0.0.1/tcp/3034/p2p/QmTBup8mUZcNkytxTgz1xWxQNPFQFh5Gnwjdy19D1BPqpd".into(),
        "/ip4/127.0.0.1/tcp/3035/p2p/QmSm4yua8ift1AULVisKooPwviVybuRFadr9Rmqurn5DWw".into(),
    ];
    ChainSpec::from_genesis(
        "DNA",
        "dna_testnet",
        dna_config_genesis,
        boot_nodes,
        Some(TelemetryEndpoints::new(vec![(
            STAGING_TELEMETRY_URL.to_string(),
            0,
        )])),
        Some(DNA_PROTOCOL_ID),
        Some(serde_json::from_str(DNA_PROPERTIES).unwrap()),
        Default::default(),
    )
}

fn development_config_genesis() -> GenesisConfig {
    testnet_genesis(vec![get_authority_keys_from_seed("Alice")], None)
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Development",
        "dev",
        development_config_genesis,
        vec![],
        None,
        Some(DNA_PROTOCOL_ID),
        Some(serde_json::from_str(DNA_PROPERTIES).unwrap()),
        Default::default(),
    )
}

fn local_testnet_genesis() -> GenesisConfig {
    testnet_genesis(
        vec![
            get_authority_keys_from_seed("Alice"),
            get_authority_keys_from_seed("Bob"),
        ],
        Some(vec![get_account_id_from_seed::<sr25519::Public>("Alice")]),
    )
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn local_testnet_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Local Testnet",
        "local_testnet",
        local_testnet_genesis,
        vec![],
        None,
        Some(DNA_PROTOCOL_ID),
        Some(serde_json::from_str(DNA_PROPERTIES).unwrap()),
        Default::default(),
    )
}
