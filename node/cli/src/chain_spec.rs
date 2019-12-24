use grandpa::AuthorityId as GrandpaId;
use babe_primitives::AuthorityId as BabeId;
use im_online::sr25519::AuthorityId as ImOnlineId;
use authority_discovery_primitives::AuthorityId as AuthorityDiscoveryId;
use sp_runtime::{Perbill, traits::{Verify, IdentifyAccount}};
use primitives::{Pair, Public, crypto::UncheckedInto, sr25519};
use node_runtime::{
    GenesisConfig, SystemConfig, SessionConfig, BabeConfig, StakingConfig,
    IndicesConfig, ImOnlineConfig, BalancesConfig, GrandpaConfig, SudoConfig,
    AuthorityDiscoveryConfig,
    SessionKeys, StakerStatus, WASM_BINARY,
};
use node_runtime::constants::currency::*;
use node_primitives::{AccountId, Balance, Signature};
use telemetry::TelemetryEndpoints;
use hex_literal::hex;
use log::info;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

// TODO: Remove if not needed
const DNA_PROTOCOL_ID: &str = "dna"; // we dont need this
const DNA_PROPERTIES: &str = r#"
    {
        "tokenDecimals": 4,
        "tokenSymbol": "DNA"
    }"#;


type AccountPublic = <Signature as Verify>::Signer;

/// Specialised `ChainSpec`. This is a specialisation of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::ChainSpec<GenesisConfig>;

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

impl ChainOpt {
    /// Get an actual chain config from one of the alternatives.
    pub(crate) fn load(self) -> Result<ChainSpec, String> {
        Ok(match self {
            ChainOpt::Development => development_config(),
            ChainOpt::LocalTestnet => local_testnet_config(),
            ChainOpt::DNATestnet => dna_testnet_config(),
        })
    }

    pub(crate) fn from(s: &str) -> Option<Self> {
        match s {
            "dev" => Some(ChainOpt::Development),
            "local" => Some(ChainOpt::LocalTestnet),
            "" | "dna" => Some(ChainOpt::DNATestnet),
            _ => None,
        }
    }
}

pub fn load_spec(id: &str) -> Result<Option<ChainSpec>, String> {
    Ok(match ChainOpt::from(id) {
        Some(spec) => Some(spec.load()?),
        None => None,
    })
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn get_authority_keys_from_seed(
    seed: &str
) -> (
    AccountId,
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId
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
    SessionKeys { grandpa, babe, im_online, authority_discovery, }
}

/// Helper function to create GenesisConfig for testing
pub fn testnet_genesis(
    initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)>,
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
        indices: Some(IndicesConfig {
            ids: endowed_accounts.iter().cloned()
                .chain(initial_authorities.iter().map(|x| x.0.clone()))
                .collect::<Vec<_>>(),
        }),
        balances: Some(BalancesConfig {
            balances: endowed_accounts.iter().cloned()
                .map(|k| (k, ENDOWMENT))
                .chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
                .collect(),
            vesting: vec![],
        }),
        session: Some(SessionConfig {
            keys: initial_authorities.iter().map(|x| {
                (x.0.clone(), session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()))
            }).collect::<Vec<_>>(),
        }),
        staking: Some(StakingConfig {
            current_era: 0,
            validator_count: 3,
            minimum_validator_count: 2,
            stakers: initial_authorities.iter().map(|x| {
                (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator)
            }).collect(),
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: Perbill::from_percent(10),
            .. Default::default()
        }),
        sudo: Some(SudoConfig {
            key: endowed_accounts[0].clone(),
        }),
        babe: Some(BabeConfig {
            authorities: vec![],
        }),
        grandpa: Some(GrandpaConfig {
            authorities: vec![],
        }),
        im_online: Some(ImOnlineConfig {
            keys: vec![],
        }),
        authority_discovery: Some(AuthorityDiscoveryConfig {
            keys: vec![],
        }),
    }
}

// DNA Testnet config for testing and development
fn dna_config_genesis() -> GenesisConfig {
    let initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)> = vec![(
        hex!["702078e8037ac21bd9dd872cf4dd87ad9c63172d24f66139ad5891412b95674f"].into(),
        hex!["702078e8037ac21bd9dd872cf4dd87ad9c63172d24f66139ad5891412b95674f"].into(),
        hex!["833c7b8896ef66f70c0ceb079e9c13772d1cdea271248ebfe0587a7fbc2ffef9"].unchecked_into(),
        hex!["8e74d42e90c1d6fa1f6bf332a0ae79e896d84fbb189160bba6e79cdbccc0224f"].unchecked_into(),
        hex!["9cfa107b702bed0e9a608dffdf9fc5b96a6a059677d393d244e8f53e79ad9e3f"].unchecked_into(),
        hex!["1c9d3d4c3079a047f83f2ddf671d76c415381cedb0b2806752b66b7dbf6d3c76"].unchecked_into(),
    ),];

    let endowed_accounts: Vec<AccountId> = vec![
        // 5EbisDGXTdMScRusn6vBZ9B5QASm5LMRCVkERjRnUpz6bpi4
        hex!["702078e8037ac21bd9dd872cf4dd87ad9c63172d24f66139ad5891412b95674f"].into(),
    ];


//    info!( "Hello --------------------------------------------------");
);
    testnet_genesis(
        initial_authorities,
        Some(endowed_accounts),
    )
}

// pub fn dna_testnet_config() -> ChainSpec {
//     ChainSpec::from_json_bytes(&include_bytes!("../res/spec.dna.json")[..]).unwrap()
// }

/// testnet config.
pub fn dna_testnet_config() -> ChainSpec {
    let boot_nodes = vec![
        // validator-01
        "/ip4/192.168.1.201/tcp/3033/p2p/QmW7EaC6puS4QRLhXZSTZUY2zgSfV2mDnaDxLmZQxs73Xm".into(),
        ];
    ChainSpec::from_genesis(
        "DNA",
        "dna_testnet",
        dna_config_genesis,
        boot_nodes,
        Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])),
        Some(DNA_PROTOCOL_ID),
        Some(serde_json::from_str(DNA_PROPERTIES).unwrap()),
        Default::default(),
    )
}

fn development_config_genesis() -> GenesisConfig {
    testnet_genesis(
        vec![
            get_authority_keys_from_seed("Alice"),
        ],
        None,
    )
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
        None,
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
