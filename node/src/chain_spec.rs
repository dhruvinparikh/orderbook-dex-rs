use crate::fixtures::*;
use babe_primitives::AuthorityId as BabeId;
use im_online::sr25519::AuthorityId as ImOnlineId;
use grandpa_primitives::AuthorityId as GrandpaId;
use primitives::{sr25519, Pair, Public};
use runtime::{
    AccountId, BabeConfig, BalancesConfig, GenesisConfig, GrandpaConfig, IndicesConfig, Signature,
    SudoConfig, SystemConfig,
    WASM_BINARY,
};
use sp_runtime::traits::{IdentifyAccount, Verify};
use std::borrow::Cow; // Used to import from json file
use substrate_service;
use substrate_telemetry::TelemetryEndpoints;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = substrate_service::ChainSpec<GenesisConfig>;

/// The chain specification option. This is expected to come in from the CLI and
/// is little more than one of a number of alternatives which can easily be converted
/// from a string (`--chain=...`) into a `ChainSpec`.
#[derive(Clone, Debug)]
pub enum Alternative {
    /// Whatever the current runtime is, with just Alice as an auth.
    Development,
    /// Whatever the current runtime is, with simple Alice/Bob auths.
    LocalTestnet,
    /// Hosted testnet with auto-generated genesis block. Use this to build-spec and
    /// generate a template for a unified genesis block.
    /// Use `dna build-spec --chain staging >> node/res/dna_raw.json` to generate
    /// Testnet chainspec json file
    /// Update name, id, properties and if necessary bootnodes
    // "properties": {
    //     "ss58Format": 7,
    //     "tokenDecimals": 9,
    //     "tokenSymbol": "XTL"
    //   },
    StagingTestnet,
    /// Hosted testnet with unified genesis block and non-standard Validators.
    Testnet,
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate an authority key for Babe
pub fn get_authority_keys_from_seed(s: &str) -> (BabeId,ImOnlineId, GrandpaId) {
    (get_from_seed::<BabeId>(s),
    get_from_seed::<ImOnlineId>(s),
    get_from_seed::<GrandpaId>(s))
}

impl Alternative {
    /// Get an actual chain config from one of the alternatives.
    pub(crate) fn load(self) -> Result<ChainSpec, String> {
        Ok(match self {
            Alternative::Development => ChainSpec::from_genesis(
                "Development",
                "dev",
                || {
                    testnet_genesis(
                        vec![get_authority_keys_from_seed("Alice")],
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        vec![
                            get_account_id_from_seed::<sr25519::Public>("Alice"),
                            get_account_id_from_seed::<sr25519::Public>("Bob"),
                            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                            get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                        ],
                        true,
                    )
                },
                vec![],
                None,
                None,
                None,
                None,
            ),
            Alternative::LocalTestnet => ChainSpec::from_genesis(
                "Local Testnet",
                "local_testnet",
                || {
                    testnet_genesis(
                        vec![
                            get_authority_keys_from_seed("Alice"),
                            get_authority_keys_from_seed("Bob"),
                        ],
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
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
                        ],
                        true,
                    )
                },
                vec![],
                None,
                None,
                None,
                None,
            ),
            Alternative::StagingTestnet => ChainSpec::from_genesis(
                "DNA Chain Staging", // Name
                "staging",             // Id
                || {
                    testnet_genesis(
                        // TODO: Replace with get_staging_initial_authorities() once key generation is fixed
                        vec![
                            get_authority_keys_from_seed("Alice"),
                            get_authority_keys_from_seed("Bob"),
                        ], // Initial Authorities
                        get_staging_root_key(),
                        get_staging_endowed_accounts(), // Endowed Accounts
                        true,
                    )
                }, // Constructor
                get_staging_bootnodes(), // Boot Nodes
                Some(TelemetryEndpoints::new(vec![(
                    STAGING_TELEMETRY_URL.to_string(),
                    0,
                )])), // Telemetry Endpoints
                None,                  // Protocol Id
                None,                  // Consensus Engine
                None,
            ),
            Alternative::Testnet => ChainSpec::from_json_bytes(Cow::Owned(
                include_bytes!("../res/dnachain_raw.json").to_vec(),
            ))
            .unwrap(),
        })
    }

    pub(crate) fn from(s: &str) -> Option<Self> {
        match s {
            "dev" => Some(Alternative::Development),
            "local" => Some(Alternative::LocalTestnet),
            "staging" => Some(Alternative::StagingTestnet),
            "" | "testnet" => Some(Alternative::Testnet),
            _ => None,
        }
    }
}

fn testnet_genesis(
    initial_authorities: Vec<(BabeId,ImOnlineId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> GenesisConfig {
    GenesisConfig {
        system: Some(SystemConfig {
            code: WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        indices: Some(IndicesConfig {
            ids: endowed_accounts.clone(),
        }),
        balances: Some(BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
            vesting: vec![],
        }),
        sudo: Some(SudoConfig { key: root_key }),
        babe: Some(BabeConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        }),
        grandpa: Some(GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        }),
    }
}
