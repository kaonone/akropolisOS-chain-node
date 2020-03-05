//! Substrate Node Template CLI library.
#![warn(missing_docs)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;

fn main() -> sc_cli::Result<()> {
	let version = sc_cli::VersionInfo {
		name: "AkropolisOS",
		commit: env!("VERGEN_SHA_SHORT"),
		version: env!("CARGO_PKG_VERSION"),
		executable_name: "akropolisos-node",
		author: "Akropolis",
		description: "Akropolis OS Node",
		support_url: "admin@akropolis.io",
		copyright_start_year: 2017,
	};

	command::run(version)
}