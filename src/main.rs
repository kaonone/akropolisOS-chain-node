//! Substrate Node CLI
#![warn(missing_docs)]

use std::env;
// use dotenv::dotenv;
fn main() -> sc_cli::Result<()> {
	// dotenv().ok();
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

	node_cli::run(env::args(), version)
}