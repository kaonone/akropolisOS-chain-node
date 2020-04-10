
use crate::chain_spec::ChainSpec;
use log::info;
use wasm_bindgen::prelude::*;
use sc_service::Configuration;
use browser_utils::{
	Client,
	browser_configuration, set_console_error_panic_hook, init_console_log,
};
use std::str::FromStr;

/// Starts the client.
#[wasm_bindgen]
pub async fn start_client(chain_spec: String, log_level: String) -> Result<Client, JsValue> {
	start_inner(chain_spec, log_level)
		.await
		.map_err(|err| JsValue::from_str(&err.to_string()))
}

async fn start_inner(chain_spec: String, log_level: String) -> Result<Client, Box<dyn std::error::Error>> {
	set_console_error_panic_hook();
	init_console_log(log::Level::from_str(&log_level)?)?;
	let chain_spec = ChainSpec::from_json_bytes(chain_spec.as_bytes().to_vec())
		.map_err(|e| format!("{:?}", e))?;

	let config = browser_configuration(chain_spec).await?;

	info!("Akropolis OS browser node");
	info!("  version {}", config.full_version());
	info!("  by Akropolis Decentralized LTD, 2017-2020");
	info!("Chain specification: {}", config.expect_chain_spec().name());
	info!("Node name: {}", config.name);
	info!("Roles: {:?}", config.roles);

	// Create the service. This is the most heavy initialization step.
	let service = crate::service::new_light(config)
		.map_err(|e| format!("{:?}", e))?;

	Ok(browser_utils::start_client(service))
}