use grandpa::AuthorityId as GrandpaId;
use babe_primitives::AuthorityId as BabeId;
use im_online::sr25519::AuthorityId as ImOnlineId;
use primitives::{ed25519, sr25519, Pair};
use dna_runtime::{
	AccountId, GenesisConfig, ConsensusConfig, TimestampConfig, BalancesConfig,
	SudoConfig, IndicesConfig, BabeConfig, StakingConfig, SessionConfig, SystemConfig, 
	ImOnlineConfig, GrandpaConfig, AuthorityDiscoveryConfig, SessionKeys, StakerStatus,
};
use substrate_service::{self, Properties};
use substrate_telemetry::TelemetryEndpoints;
use serde_json::json;
use dna_runtime::constants::currency::*;
use dna_runtime::types::{AccountId, Balance};
use hex_literal::hex;

use ed25519::Public as AuthorityId;

// Note this is the URL for the telemetry server
const STAGING_TELEMETRY_URL: &str = " ws://localhost:1024";

const PRODUCTION_TELEMETRY_URL: &str = " ws://localhost:1024";

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
	ProductionTestnet
}

fn authority_key(s: &str) -> AuthorityId {
	ed25519::Pair::from_string(&format!("{}", s), None)
		.expect("static values are valid; qed")
		.public()
}

fn account_key(s: &str) -> AccountId {
	sr25519::Pair::from_string(&format!("{}", s), None)
		.expect("static values are valid; qed")
		.public()
}

fn authority_key_dev(s: &str) -> AuthorityId {
	ed25519::Pair::from_string(&format!("//{}", s), None)
		.expect("static values are valid; qed")
		.public()
}

fn account_key_dev(s: &str) -> AccountId {
	sr25519::Pair::from_string(&format!("//{}", s), None)
		.expect("static values are valid; qed")
		.public()
}

fn session_keys(grandpa: GrandpaId, babe: BabeId, im_online: ImOnlineId) -> SessionKeys {
    SessionKeys { grandpa, babe, im_online, }
}

fn dna_props() -> Properties {
	json!({"tokenDecimals": 4, "tokenSymbol": "DNA" }).as_object().unwrap().clone()
}
impl Alternative {
	/// Get an actual chain config from one of the alternatives.
	pub(crate) fn load(self) -> Result<ChainSpec, String> {
		Ok(match self {
			Alternative::Development => ChainSpec::from_genesis(
				"Development",
				"dev",
				|| testnet_genesis(vec![
	        authority_key_dev("Alice"),
          authority_key_dev("Bob"),
					authority_key_dev("Alice"),
					authority_key_dev("Alice"),
					authority_key_dev("Alice"),
				], vec![
					account_key_dev("Alice")
				],
					account_key_dev("Alice")
				),
				vec![],
				Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])),	
				None,
				None,
				Some(dna_props())
			),
			Alternative::LocalTestnet => ChainSpec::from_genesis(
				"Local Testnet",
				"local_testnet",
				|| testnet_genesis(vec![
					authority_key_dev("Alice"),
          authority_key_dev("Bob"),
					authority_key_dev("Alice"),
					authority_key_dev("Alice"),
					authority_key_dev("Alice"),


				], vec![
					account_key_dev("Alice"),
					account_key_dev("Bob"),
					account_key_dev("Charlie"),
					account_key_dev("Dave"),
					account_key_dev("Eve"),
					account_key_dev("Ferdie"),
				],
					account_key_dev("Alice"),
				),
				vec![],
				Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])),	
				None,
				None,
				Some(dna_props())

			),
			Alternative::ProductionTestnet => ChainSpec::from_genesis(
				"Production Testnet",
				"Production_testnet",
				|| production_genesis(vec![
					authority_key("budget number index moon visa midnight process answer large panther tenant appear"),
					authority_key("coconut session disorder bone spot tattoo uncover basket basic laundry glad shiver"),
					authority_key("debate convince invite virus shy tank swift fuel aerobic open alien address"),


				], vec![
					account_key("budget number index moon visa midnight process answer large panther tenant appear"),
					account_key("coconut session disorder bone spot tattoo uncover basket basic laundry glad shiver"),
					account_key("debate convince invite virus shy tank swift fuel aerobic open alien address"),


				],
					account_key("budget number index moon visa midnight process answer large panther tenant appear"),

				),
				vec![],
				Some(TelemetryEndpoints::new(vec![(PRODUCTION_TELEMETRY_URL.to_string(), 0)])),	
				None,
				None,
				Some(dna_props())

			),
		})
	}

	pub(crate) fn from(s: &str) -> Option<Self> {
		match s {
			"dev" => Some(Alternative::Development),
			"" | "local" => Some(Alternative::LocalTestnet),
			"prod" => Some(Alternative::ProductionTestnet),
			_ => None,
		}
	}
}

fn testnet_genesis(initial_authorities: Vec<(AuthorityId, AuthorityId, GrandpaId, BabeId, ImOnlineId)>, endowed_accounts: Vec<AccountId>, root_key: AccountId) -> GenesisConfig {
	GenesisConfig {

    system: Some(SystemConfig {
			      code: include_bytes!("../runtime/wasm/target/wasm32-unknown-unknown/release/dna_runtime_wasm.compact.wasm").to_vec(),
            changes_trie_config: Default::default(),
        }),
		timestamp: Some(TimestampConfig {
			minimum_period: 5, // 10 second block time.
		}),
		indices: Some(IndicesConfig {
			ids: endowed_accounts.clone(),
		}),
		balances: Some(BalancesConfig {
			transaction_base_fee: 1,
			transaction_byte_fee: 0,
			existential_deposit: 500,
			transfer_fee: 0,
			creation_fee: 0,
			balances: endowed_accounts.iter().cloned().map(|k|(k, 1 << 60)).collect(),
			vesting: vec![],
		}),
    session: Some(SessionConfig {
            keys: initial_authorities.iter().map(|x| {
                (x.1.clone(), session_keys(x.2.clone(), x.3.clone(), x.4.clone()))
            }).collect::<Vec<_>>(),
        }),
        staking: Some(StakingConfig {
            current_era: 0,
            validator_count: 10,
            minimum_validator_count: 3,
            stakers: initial_authorities.iter().map(|x| {
                (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator)
            }).collect(),
            invulnerables: initial_authorities.iter().map(|x| x.1.clone()).collect(),
            slash_reward_fraction: Perbill::from_percent(10),
            .. Default::default()
        }),
		sudo: Some(SudoConfig {
			key: root_key,
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
        authority_discovery: Some(AuthorityDiscoveryConfig{
            keys: vec![],
        }),
	}
}

fn production_genesis(initial_authorities: Vec<AuthorityId>, endowed_accounts: Vec<AccountId>, root_key: AccountId) -> GenesisConfig {
	GenesisConfig {
		consensus: Some(ConsensusConfig {
			code: include_bytes!("../runtime/wasm/target/wasm32-unknown-unknown/release/dna_runtime_wasm.compact.wasm").to_vec(),
			authorities: initial_authorities.clone(),
		}),
		system: None,
		timestamp: Some(TimestampConfig {
			minimum_period: 5, // 10 second block time.
		}),
		indices: Some(IndicesConfig {
			ids: endowed_accounts.clone(),
		}),
		balances: Some(BalancesConfig {
			transaction_base_fee: 1,
			transaction_byte_fee: 0,
			existential_deposit: 500,
			transfer_fee: 0,
			creation_fee: 0,
			balances: endowed_accounts.iter().cloned().map(|k|(k, 1 << 60)).collect(),
			vesting: vec![],
		}),
		sudo: Some(SudoConfig {
			key: root_key,
		}),
	}
}
