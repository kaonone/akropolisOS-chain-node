//! The Substrate Node Template runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit="256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use sp_std::prelude::*;
use sp_core::OpaqueMetadata;
use sp_runtime::{
	ApplyExtrinsicResult, transaction_validity::TransactionValidity, generic, create_runtime_str,
	impl_opaque_keys, MultiSignature
};
use sp_runtime::traits::{
	BlakeTwo256, Block as BlockT, StaticLookup, Verify, ConvertInto, IdentifyAccount
};
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use grandpa::AuthorityList as GrandpaAuthorityList;
use grandpa::fg_primitives;
use sp_version::RuntimeVersion;
#[cfg(feature = "std")]
use sp_version::NativeVersion;

// A few exports that help ease life for downstream crates.
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use bridge::Call as BridgeCall;
pub use timestamp::Call as TimestampCall;
pub use balances::Call as BalancesCall;
pub use sp_runtime::{Permill, Perbill};
pub use frame_support::{
	StorageValue, construct_runtime, parameter_types,
	traits::{Randomness, Get},
	weights::Weight,
};
// use contracts::Gas;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

pub mod bridge;
mod dao;
mod marketplace;
mod token;
pub mod types;
mod price_fetch;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core datastructures.
pub mod opaque {
	use super::*;
	
	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
	
	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
	
	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
			pub grandpa: Grandpa,
		}
	}
}

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("node-template"),
	impl_name: create_runtime_str!("node-template"),
	authoring_version: 1,
	spec_version: 1,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
};

pub const MILLISECS_PER_BLOCK: u64 = 6000;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// These time units are defined in number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

/// The version infromation used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

parameter_types! {
	pub const BlockHashCount: BlockNumber = 250;
	pub const MaximumBlockWeight: Weight = 1_000_000;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
	pub const Version: RuntimeVersion = VERSION;
}

impl system::Trait for Runtime {
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = Indices;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type Event = Event;
	/// The ubiquitous origin type.
	type Origin = Origin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// Maximum weight of each block.
	type MaximumBlockWeight = MaximumBlockWeight;
	/// Maximum size of all encoded transactions (in bytes) that are allowed in one block.
	type MaximumBlockLength = MaximumBlockLength;
	/// Portion of the block weight that is available to all normal transactions.
	type AvailableBlockRatio = AvailableBlockRatio;
	/// Version of the runtime.
	type Version = Version;
	/// Converts a module to the index of the module in `construct_runtime!`.
	///
	/// This type is being generated by `construct_runtime!`.
	type ModuleToIndex = ModuleToIndex;
}

impl aura::Trait for Runtime {
	type AuthorityId = AuraId;
}

impl grandpa::Trait for Runtime {
	type Event = Event;
}

impl indices::Trait for Runtime {
	/// The type for recording indexing into the account enumeration. If this ever overflows, there
	/// will be problems!
	type AccountIndex = AccountIndex;
	/// Use the standard means of resolving an index hint from an id.
	type ResolveHint = indices::SimpleResolveHint<Self::AccountId, Self::AccountIndex>;
	/// Determine whether an account is dead.
	type IsDeadAccount = Balances;
	/// The ubiquitous event type.
	type Event = Event;
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl timestamp::Trait for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = MinimumPeriod;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
	pub const TransferFee: u128 = 0;
	pub const CreationFee: u128 = 0;
}

impl balances::Trait for Runtime {
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// What to do if an account's free balance gets zeroed.
	type OnFreeBalanceZero = ();
	/// What to do if an account gets reaped.
	type OnReapAccount = ();
	/// What to do if a new account is created.
	type OnNewAccount = Indices;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type TransferPayment = ();
	type ExistentialDeposit = ExistentialDeposit;
	type TransferFee = TransferFee;
	type CreationFee = CreationFee;
}

parameter_types! {
	pub const TransactionBaseFee: Balance = 0;
	pub const TransactionByteFee: Balance = 1;
}

impl transaction_payment::Trait for Runtime {
	type Currency = balances::Module<Runtime>;
	type OnTransactionPayment = ();
	type TransactionBaseFee = TransactionBaseFee;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = ConvertInto;
	type FeeMultiplierUpdate = ();
}

impl sudo::Trait for Runtime {
	type Event = Event;
	type Proposal = Call;
}



// type TechnicalCollective = pallet_collective::Instance2;
// impl pallet_collective::Trait<TechnicalCollective> for Runtime {
// 	type Origin = Origin;
// 	type Proposal = Call;
// 	type Event = Event;
// }

// parameter_types! {
// 	pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
// }

// impl session::Trait for Runtime {
// 	type OnSessionEnding = Staking;
// 	type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
// 	type ShouldEndSession = Babe;
// 	type Event = Event;
// 	type Keys = SessionKeys;
// 	type ValidatorId = <Self as frame_system::Trait>::AccountId;
// 	type ValidatorIdOf = pallet_staking::StashOf<Self>;
// 	type SelectInitialValidators = Staking;
// 	type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
// }

// impl session::historical::Trait for Runtime {
// 	type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
// 	type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
// }

// pallet_staking_reward_curve::build! {
// 	const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
// 		min_inflation: 0_025_000,
// 		max_inflation: 0_100_000,
// 		ideal_stake: 0_500_000,
// 		falloff: 0_050_000,
// 		max_piece_count: 40,
// 		test_precision: 0_005_000,
// 	);
// }
// pub struct CurrencyToVoteHandler;

// impl CurrencyToVoteHandler {
//     fn factor() -> u128 {
//         (Balances::total_issuance() / u128::from(u64::max_value())).max(1)
//     }
// }

// impl Convert<u128, u64> for CurrencyToVoteHandler {
//     fn convert(x: u128) -> u64 {
//         (x / Self::factor()) as u64
//     }
// }

// impl Convert<u128, u128> for CurrencyToVoteHandler {
//     fn convert(x: u128) -> u128 {
//         x * Self::factor()
//     }
// }

// parameter_types! {
// 	pub const SessionsPerEra: sp_staking::SessionIndex = 6;
// 	pub const BondingDuration: pallet_staking::EraIndex = 24 * 28;
// 	pub const SlashDeferDuration: pallet_staking::EraIndex = 24 * 7; // 1/4 the bonding duration.
// 	pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
// }

// impl staking::Trait for Runtime {
// 	type Currency = Balances;
// 	type Time = Timestamp;
// 	type CurrencyToVote = CurrencyToVoteHandler;
// 	type RewardRemainder = Treasury;
// 	type Event = Event;
// 	type Slash = Treasury; // send the slashed funds to the treasury.
// 	type Reward = (); // rewards are minted from the void
// 	type SessionsPerEra = SessionsPerEra;
// 	type BondingDuration = BondingDuration;
// 	type SlashDeferDuration = SlashDeferDuration;
// 	/// A super-majority of the council can cancel the slash.
// 	type SlashCancelOrigin = pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, CouncilCollective>;
// 	type SessionInterface = Self;
// 	type RewardCurve = RewardCurve;
// }

// type CouncilCollective = pallet_collective::Instance1;
// impl pallet_collective::Trait<CouncilCollective> for Runtime {
// 	type Origin = Origin;
// 	type Proposal = Call;
// 	type Event = Event;
// }

// parameter_types! {
// 	pub const LaunchPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
// 	pub const VotingPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
// 	pub const EmergencyVotingPeriod: BlockNumber = 3 * 24 * 60 * MINUTES;
// 	pub const MinimumDeposit: Balance = 100 * DOLLARS;
// 	pub const EnactmentPeriod: BlockNumber = 30 * 24 * 60 * MINUTES;
// 	pub const CooloffPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
// 	// One cent: $10,000 / MB
// 	pub const PreimageByteDeposit: Balance = 1 * CENTS;
// }

// impl democracy::Trait for Runtime {
// 	type Proposal = Call;
// 	type Event = Event;
// 	type Currency = Balances;
// 	type EnactmentPeriod = EnactmentPeriod;
// 	type LaunchPeriod = LaunchPeriod;
// 	type VotingPeriod = VotingPeriod;
// 	type EmergencyVotingPeriod = EmergencyVotingPeriod;
// 	type MinimumDeposit = MinimumDeposit;
// 	/// A straight majority of the council can decide what their next motion is.
// 	type ExternalOrigin = pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>;
// 	/// A super-majority can have the next scheduled referendum be a straight majority-carries vote.
// 	type ExternalMajorityOrigin = pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, CouncilCollective>;
// 	/// A unanimous council can have the next scheduled referendum be a straight default-carries
// 	/// (NTB) vote.
// 	type ExternalDefaultOrigin = pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollective>;
// 	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
// 	/// be tabled immediately and with a shorter voting/enactment period.
// 	type FastTrackOrigin = pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, TechnicalCollective>;
// 	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
// 	type CancellationOrigin = pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, CouncilCollective>;
// 	// Any single technical committee member may veto a coming council proposal, however they can
// 	// only do it once and it lasts only for the cooloff period.
// 	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
// 	type CooloffPeriod = CooloffPeriod;
// 	type PreimageByteDeposit = PreimageByteDeposit;
// 	type Slash = Treasury;
// }

// parameter_types! {
// 	pub const ProposalBond: Permill = Permill::from_percent(5);
// 	pub const ProposalBondMinimum: Balance = 1 * DOLLARS;
// 	pub const SpendPeriod: BlockNumber = 1 * DAYS;
// 	pub const Burn: Permill = Permill::from_percent(50);
// }

// impl pallet_treasury::Trait for Runtime {
// 	type Currency = Balances;
// 	type ApproveOrigin = pallet_collective::EnsureMembers<_4, AccountId, CouncilCollective>;
// 	type RejectOrigin = pallet_collective::EnsureMembers<_2, AccountId, CouncilCollective>;
// 	type Event = Event;
// 	type ProposalRejection = ();
// 	type ProposalBond = ProposalBond;
// 	type ProposalBondMinimum = ProposalBondMinimum;
// 	type SpendPeriod = SpendPeriod;
// 	type Burn = Burn;
// }

// parameter_types! {
// 	pub const ContractTransferFee: Balance = 1 * CENTS;
// 	pub const ContractCreationFee: Balance = 1 * CENTS;
// 	pub const ContractTransactionBaseFee: Balance = 1 * CENTS;
// 	pub const ContractTransactionByteFee: Balance = 10 * MILLICENTS;
// 	pub const ContractFee: Balance = 1 * CENTS;
// 	pub const TombstoneDeposit: Balance = 1 * DOLLARS;
// 	pub const RentByteFee: Balance = 1 * DOLLARS;
// 	pub const RentDepositOffset: Balance = 1000 * DOLLARS;
// 	pub const SurchargeReward: Balance = 150 * DOLLARS;
// }

// impl pallet_contracts::Trait for Runtime {
// 	type Currency = Balances;
// 	type Time = Timestamp;
// 	type Randomness = RandomnessCollectiveFlip;
// 	type Call = Call;
// 	type Event = Event;
// 	type DetermineContractAddress = pallet_contracts::SimpleAddressDeterminator<Runtime>;
// 	type ComputeDispatchFee = pallet_contracts::DefaultDispatchFeeComputor<Runtime>;
// 	type TrieIdGenerator = pallet_contracts::TrieIdFromParentCounter<Runtime>;
// 	type GasPayment = ();
// 	type RentPayment = ();
// 	type SignedClaimHandicap = pallet_contracts::DefaultSignedClaimHandicap;
// 	type TombstoneDeposit = TombstoneDeposit;
// 	type StorageSizeOffset = pallet_contracts::DefaultStorageSizeOffset;
// 	type RentByteFee = RentByteFee;
// 	type RentDepositOffset = RentDepositOffset;
// 	type SurchargeReward = SurchargeReward;
// 	type TransferFee = ContractTransferFee;
// 	type CreationFee = ContractCreationFee;
// 	type TransactionBaseFee = ContractTransactionBaseFee;
// 	type TransactionByteFee = ContractTransactionByteFee;
// 	type ContractFee = ContractFee;
// 	type CallBaseFee = pallet_contracts::DefaultCallBaseFee;
// 	type InstantiateBaseFee = pallet_contracts::DefaultInstantiateBaseFee;
// 	type MaxDepth = pallet_contracts::DefaultMaxDepth;
// 	type MaxValueSize = pallet_contracts::DefaultMaxValueSize;
// 	type BlockGasLimit = pallet_contracts::DefaultBlockGasLimit;
// }

impl dao::Trait for Runtime {
    type Event = Event;
}

impl marketplace::Trait for Runtime {
    type Event = Event;
}

impl token::Trait for Runtime {
    type Event = Event;
}

impl bridge::Trait for Runtime {
    type Event = Event;
}

/// We need to define the Transaction signer for that using the Key definition
type SubmitUnsignedPFTransaction = system::offchain::TransactionSubmitter<
	price_fetch::crypto::Public,
	Runtime,
	UncheckedExtrinsic
>;
type SubmitSignedPFTransaction = system::offchain::TransactionSubmitter<
	price_fetch::crypto::Public,
	Runtime,
	UncheckedExtrinsic
>;

parameter_types! {
	pub const BlockFetchPeriod: BlockNumber = 2;
}

impl price_fetch::Trait for Runtime {
	type Event = Event;
	type Call = Call;
	type SubmitUnsignedTransaction = SubmitUnsignedPFTransaction;
	type SubmitSignedTransaction = SubmitSignedPFTransaction;
	type BlockFetchPeriod = BlockFetchPeriod;
}


impl system::offchain::CreateTransaction<Runtime, UncheckedExtrinsic> for Runtime {
	type Public = <Signature as Verify>::Signer;
	type Signature = Signature;

	fn create_transaction<TSigner: system::offchain::Signer<Self::Public, Self::Signature>> (
		call: Call,
		public: Self::Public,
		account: AccountId,
		index: Index,
	) -> Option<(Call, <UncheckedExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload)> {
		let period = 1 << 8;
		let current_block = System::block_number().into();
		let tip = 0;
		let extra: SignedExtra = (
			system::CheckVersion::<Runtime>::new(),
			system::CheckGenesis::<Runtime>::new(),
			system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
			system::CheckNonce::<Runtime>::from(index),
			system::CheckWeight::<Runtime>::new(),
			transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
		);
		let raw_payload = SignedPayload::new(call, extra).ok()?;
		let signature = TSigner::sign(public, &raw_payload)?;
		let address = Indices::unlookup(account);
		let (call, extra, _) = raw_payload.deconstruct();
		Some((call, (address, signature, extra)))
	}
}

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: system::{Module, Call, Storage, Config, Event},
		Timestamp: timestamp::{Module, Call, Storage, Inherent},
		Aura: aura::{Module, Config<T>, Inherent(Timestamp)},
		Grandpa: grandpa::{Module, Call, Storage, Config, Event},
		Indices: indices,
		Balances: balances,
		TransactionPayment: transaction_payment::{Module, Storage},
		Sudo: sudo,
		// Session: session::{Module, Call, Storage, Config<T>, Event},
        // Staking: staking,
        // Democracy: democracy,
		// Contract: contracts::{Module, Call, Config, Event<T>},
		Dao: dao::{Module, Call, Storage, Config, Event<T>},
		Marketplace: marketplace::{Module, Call, Storage, Event<T>},
        Token: token::{Module, Call, Storage, Config, Event<T>},
        Bridge: bridge::{Module, Call, Storage, Config<T>, Event<T>},
		RandomnessCollectiveFlip: randomness_collective_flip::{Module, Call, Storage},
		// Price Oracle
		PriceFetch: price_fetch::{Module, Call, Storage, Event<T>, ValidateUnsigned},
	}
);

/// The address format for describing accounts.
pub type Address = <Indices as StaticLookup>::Source;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	system::CheckVersion<Runtime>,
	system::CheckGenesis<Runtime>,
	system::CheckEra<Runtime>,
	system::CheckNonce<Runtime>,
	system::CheckWeight<Runtime>,
	transaction_payment::ChargeTransactionPayment<Runtime>
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Just that the Signature Signer needs this additional definition as well
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<Runtime, Block, system::ChainContext<Runtime>, Runtime, AllModules>;

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			Runtime::metadata().into()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}

		fn random_seed() -> <Block as BlockT>::Hash {
			RandomnessCollectiveFlip::random_seed()
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
			Executive::validate_transaction(tx)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> u64 {
			Aura::slot_duration()
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities()
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}
	}

	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}
	}
}
