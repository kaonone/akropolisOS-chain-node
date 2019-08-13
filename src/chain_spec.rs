use primitives::{ed25519, sr25519, Pair};
use akropolisos_substrate_node_runtime::{
	AccountId, Perbill, Permill, Schedule, GenesisConfig, ConsensusConfig, TimestampConfig, BalancesConfig,
	SudoConfig, IndicesConfig, SessionConfig, StakingConfig, DemocracyConfig, CouncilVotingConfig,
	TreasuryConfig, ContractConfig
};
use substrate_service;

use ed25519::Public as AuthorityId;

// Note this is the URL for the telemetry server
//const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

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
}

fn authority_key(s: &str) -> AuthorityId {
	ed25519::Pair::from_string(&format!("//{}", s), None)
		.expect("static values are valid; qed")
		.public()
}

fn account_key(s: &str) -> AccountId {
	sr25519::Pair::from_string(&format!("//{}", s), None)
		.expect("static values are valid; qed")
		.public()
}

impl Alternative {
	/// Get an actual chain config from one of the alternatives.
	pub(crate) fn load(self) -> Result<ChainSpec, String> {
		Ok(match self {
			Alternative::Development => ChainSpec::from_genesis(
				"Development",
				"dev",
				|| testnet_genesis(vec![
					authority_key("Alice")
				], vec![
					account_key("Alice")
				],
					account_key("Alice")
				),
				vec![],
				None,
				None,
				None,
				None
			),
			Alternative::LocalTestnet => ChainSpec::from_genesis(
				"Local Testnet",
				"local_testnet",
				|| testnet_genesis(vec![
					authority_key("Alice"),
					authority_key("Bob"),
				], vec![
					account_key("Alice"),
					account_key("Bob"),
					account_key("Charlie"),
					account_key("Dave"),
					account_key("Eve"),
					account_key("Ferdie"),
				],
					account_key("Alice"),
				),
				vec![],
				None,
				None,
				None,
				None
			),
		})
	}

	pub(crate) fn from(s: &str) -> Option<Self> {
		match s {
			"dev" => Some(Alternative::Development),
			"" | "local" => Some(Alternative::LocalTestnet),
			_ => None,
		}
	}
}

fn testnet_genesis(initial_authorities: Vec<AuthorityId>, endowed_accounts: Vec<AccountId>, root_key: AccountId) -> GenesisConfig {
	GenesisConfig {
		consensus: Some(ConsensusConfig {
			code: include_bytes!("../runtime/wasm/target/wasm32-unknown-unknown/release/akropolisos_substrate_node_runtime_wasm.compact.wasm").to_vec(),
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
		session: Some(SessionConfig {
			validators: endowed_accounts.clone(),
			keys: endowed_accounts.iter().cloned().zip(initial_authorities).collect(),
			session_length: 6
		}),
		staking: Some(StakingConfig {
			validator_count: 5, // The ideal number of staking participants.
			minimum_validator_count: 1, // Minimum number of staking participants before emergency conditions are imposed
			sessions_per_era: 5, // The length of a staking era in sessions.
			session_reward: Perbill::from_millionths(10_000), // Maximum reward, per validator, that is provided per acceptable session.
			offline_slash: Perbill::from_percent(50_000), // Slash, per validator that is taken for the first time they are found to be offline.
			offline_slash_grace: 3, // Number of instances of offline reports before slashing begins for validators.
			bonding_duration: 30, // The length of the bonding duration in blocks.
			invulnerables: vec![], // Any validators that may never be slashed or forcibly kicked.
			stakers: vec![], // This is keyed by the stash account.
			current_era: 0, // The current era index.
			current_session_reward: 10, // Maximum reward, per validator, that is provided per acceptable session.
		}),
		democracy: Some(DemocracyConfig {
			launch_period: 1440, // How often (in blocks) new public referenda are launched.
			minimum_deposit: 10_000, // The minimum amount to be used as a deposit for a public referendum proposal.
			public_delay: 5, // The delay before enactment for all public referenda.
			max_lock_periods: 60, // The maximum number of additional lock periods a voter may offer to strengthen their vote.
			voting_period: 144, // How often (in blocks) to check for new votes.
		}),
		council_voting: Some(CouncilVotingConfig {
			cooloff_period: 360, // Period (in blocks) that a veto is in effect.
			voting_period: 60, // Period (in blocks) that a vote is open for.
			enact_delay_period: 5, // Number of blocks by which to delay enactment of successful.
		}),
		treasury: Some(TreasuryConfig {
			proposal_bond: Permill::from_millionths(50_000), // Proportion of funds that should be bonded in order to place a proposal.
			proposal_bond_minimum: 1_000_000, // Minimum amount of funds that should be placed in a deposit for making a proposal.
			spend_period: 360, // Period between successive spends.
			burn: Permill::from_millionths(100_000), // Percentage of spare funds (if any) that are burnt per spend period.
		}),
		contract: Some(ContractConfig {
			transfer_fee: 100, // The fee required to make a transfer.
			creation_fee: 100, // The fee required to create an account.
			transaction_base_fee: 21, // The fee to be paid for making a transaction; the base.
			transaction_byte_fee: 1, // The fee to be paid for making a transaction; the per-byte portion.
			contract_fee: 21, // The fee required to create a contract instance.
			call_base_fee: 135, // The base fee charged for calling into a contract.
			create_base_fee: 175, // The base fee charged for creating a contract.
			gas_price: 1, // The price of one unit of gas.
			max_depth: 100, // The maximum nesting level of a call/create stack.
			block_gas_limit: 10_000_000, // The maximum amount of gas that could be expended per block.
			current_schedule: Schedule::default(), // Current cost schedule for contracts.
		}),
	}
}
