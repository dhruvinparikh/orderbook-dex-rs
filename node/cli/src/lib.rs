//! Console line interface.

// TODO : KP : Remove code start.
// use log::info;
// pub use sc_cli::VersionInfo;
// use sc_cli::{error, IntoExit};
// use sc_cli::{parse_and_prepare, NoCustom, ParseAndPrepare};
// use sc_service::{AbstractService, Configuration, Roles as ServiceRoles};
// use tokio::prelude::Future;
// use tokio::runtime::{Builder as RuntimeBuilder, Runtime};
// TODO : KP : Remove code end.
#![warn(missing_docs)]

pub mod chain_spec;

#[macro_use]
mod service;
#[cfg(feature = "browser")]
mod browser;
#[cfg(feature = "cli")]
mod cli;
#[cfg(feature = "cli")]
mod command;
#[cfg(feature = "cli")]
mod factory_impl;

#[cfg(feature = "browser")]
pub use browser::*;
#[cfg(feature = "cli")]
pub use cli::*;
#[cfg(feature = "cli")]
pub use command::*;

#[derive(Clone, Debug, PartialEq)]
pub enum ChainSpec {
    Development,
    LocalTestnet,
    DNATestnet,
}

impl ChainSpec {
    /// Get an actual chain config from one of the alternatives.
    pub(crate) fn load(self) -> Result<chain_spec::ChainSpec, String> {
        Ok(match self {
            ChainSpec::Development => chain_spec::development_config(),
            ChainSpec::LocalTestnet => chain_spec::local_testnet_config(),
            ChainSpec::DNATestnet => chain_spec::dna_testnet_config(),
        })
    }

    pub(crate) fn from(s: &str) -> Option<Self> {
        match s {
            "dev" => Some(ChainSpec::Development),
            "local" => Some(ChainSpec::LocalTestnet),
            "" | "dna" => Some(ChainSpec::DNATestnet),
            _ => None,
        }
    }
}

fn load_spec(id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
    Ok(match ChainSpec::from(id) {
        Some(spec) => Box::new(spec.load()?),
        None => Box::new(chain_spec::ChainSpec::from_json_file(
            std::path::PathBuf::from(id),
        )?),
    })
}
