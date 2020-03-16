#![warn(missing_docs)]

fn main() -> sc_cli::Result<()> {
    let version = sc_cli::VersionInfo {
        name: "DNA Node",
        commit: env!("VERGEN_SHA_SHORT"),
        version: env!("CARGO_PKG_VERSION"),
        executable_name: "dnachain",
        author: "BlockX Labs <infon@blockxlabs.com>",
        description: "DNA Pre-Mainnet Node",
        support_url: "https://github.com/blockxlabs/substrate/issues/new",
        copyright_start_year: 2019,
    };

    node_cli::run(std::env::args(), version)
}
