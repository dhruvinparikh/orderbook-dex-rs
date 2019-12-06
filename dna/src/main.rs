//! DNA Chain Blockchain CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod fixtures;

pub use sc_cli::{error, IntoExit, VersionInfo};

fn main() -> Result<(), cli::error::Error> {
    let version = VersionInfo {
        name: "DNA Chain Blockchain",
        commit: env!("VERGEN_SHA_SHORT"),
        version: env!("CARGO_PKG_VERSION"),
        executable_name: "dnachain",
        author: "kpatel, dhruvin, jesse",
        description: "Cryptocurrency Movement, Standardized.",
        support_url: "https://github.com/blockxlabs/dna/issues/new",
    };

    cli::run(std::env::args(), cli::Exit, version)
}
