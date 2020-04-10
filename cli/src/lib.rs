//! Substrate CLI library.
//!
//! This package has two Cargo features:
//!
//! - `cli` (default): exposes functions that parse command-line options, then start and run the
//! node as a CLI application.
//!
//! - `browser`: exposes the content of the `browser` module, which consists of exported symbols
//! that are meant to be passed through the `wasm-bindgen` utility and called from JavaScript.
//! Despite its name the produced WASM can theoretically also be used from NodeJS, although this
//! hasn't been tested.

pub mod chain_spec;

#[macro_use]
pub mod service;
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

/// The chain specification option.
#[derive(Clone, Debug, PartialEq)]
pub enum ChainSpec {
    /// Whatever the current runtime is, with just Alice as an auth.
    Development,
    /// Whatever the current runtime is, with simple Alice/Bob auths.
    LocalTestnet,
    /// Whatever the current runtime is with the "global testnet" defaults.
    AkropolisOSStaging,
    /// Syracuse testnet
    AkropolisOSSyracuse,
    /// Akropolis OS Mainnet
    AkropolisOS,
}

/// Get a chain config from a spec setting.
impl ChainSpec {
    pub(crate) fn load(self) -> Result<chain_spec::ChainSpec, String> {
        Ok(match self {
            ChainSpec::Development => chain_spec::development_config(),
            ChainSpec::LocalTestnet => chain_spec::local_testnet_config(),
            ChainSpec::AkropolisOSSyracuse => chain_spec::syracuse_testnet_config()?,
            ChainSpec::AkropolisOSStaging => chain_spec::staging_testnet_config(),
            ChainSpec::AkropolisOS => chain_spec::akropolisos_config()?,
        })
    }

    pub(crate) fn from(s: &str) -> Option<Self> {
        match s {
            "dev" => Some(ChainSpec::Development),
            "local" => Some(ChainSpec::LocalTestnet),
            "syracuse" => Some(ChainSpec::AkropolisOSSyracuse),
            "" | "akro" | "akropolisos" => Some(ChainSpec::AkropolisOS),
            "staging" => Some(ChainSpec::AkropolisOSStaging),
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

pub fn run_cli() -> sc_cli::Result<()> {
    use std::env;
    let version = sc_cli::VersionInfo {
        name: "AkropolisOS",
        commit: env!("VERGEN_SHA_SHORT"),
        version: env!("CARGO_PKG_VERSION"),
        executable_name: "akropolisos-node",
        author: "Akropolis",
        description: "Akropolis OS Node",
        support_url: "admin@akropolis.io",
        copyright_start_year: 2019,
    };

    crate::run(env::args(), version)
}
