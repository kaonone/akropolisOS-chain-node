//! The Substrate Node Template runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

use aura_primitives::sr25519::AuthorityId as AuraId;
use grandpa::fg_primitives;
use grandpa::AuthorityList as GrandpaAuthorityList;
use primitives::{ed25519, sr25519, OpaqueMetadata};
use rstd::prelude::*;
use sp_api::impl_runtime_apis;
use sp_runtime::traits::{
    BlakeTwo256, Block as BlockT, Convert, IdentifyAccount, NumberFor, StaticLookup, Verify,
};
use sp_runtime::{
    self, create_runtime_str, generic, impl_opaque_keys, transaction_validity::TransactionValidity,
    ApplyExtrinsicResult, MultiSignature, Perbill,
};
#[cfg(feature = "std")]
use version::NativeVersion;
use version::RuntimeVersion;

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
#[cfg(feature = "std")]
pub use staking::StakerStatus;
pub use support::{
    construct_runtime, parameter_types,
    traits::{Currency, Get},
    weights::Weight,
    StorageValue,
};

pub use balances::Call as BalancesCall;
pub use bridge::Call as BridgeCall;
use contracts::Gas;
pub use timestamp::Call as TimestampCall;

/// The type that is used for identifying authorities.
pub type AuthorityId = <AuthoritySignature as Verify>::Signer;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Index of a transaction in the chain.
pub type Index = u32;

/// The type used by authorities to prove their ID.
pub type AuthoritySignature = ed25519::Signature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// The type used by authorities to prove their ID.
pub type AccountSignature = sr25519::Signature;

/// A hash of some data used by the chain.
pub type Hash = primitives::H256;

/// Index of a block number in the chain.
pub type BlockNumber = u64;

/// Index of an account's extrinsic in the chain.
pub type Nonce = u64;

/// Balance of an account.
pub type Balance = u128;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

pub mod bridge;
mod dao;
mod marketplace;
mod token;
pub mod types;

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
    spec_name: create_runtime_str!("akropolisos-substrate-node"),
    impl_name: create_runtime_str!("akropolisos-substrate-node"),
    authoring_version: 3,
    spec_version: 4,
    impl_version: 4,
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

impl sudo::Trait for Runtime {
    /// The uniquitous event type.
    type Event = Event;
    type Proposal = Call;
}

impl session::Trait for Runtime {
    type OnSessionEnding = Staking;
    type Event = Event;
}

pub struct CurrencyToVoteHandler;

impl CurrencyToVoteHandler {
    fn factor() -> u128 {
        (Balances::total_issuance() / u128::from(u64::max_value())).max(1)
    }
}

impl Convert<u128, u64> for CurrencyToVoteHandler {
    fn convert(x: u128) -> u64 {
        (x / Self::factor()) as u64
    }
}

impl Convert<u128, u128> for CurrencyToVoteHandler {
    fn convert(x: u128) -> u128 {
        x * Self::factor()
    }
}

impl staking::Trait for Runtime {
    type Currency = balances::Module<Self>;
    type CurrencyToVote = CurrencyToVoteHandler;
    type Event = Event;
    type Slash = ();
    type Reward = ();
}

impl democracy::Trait for Runtime {
    type Currency = balances::Module<Self>;
    type Proposal = Call;
    type Event = Event;
}

impl contracts::Trait for Runtime {
    type Currency = balances::Module<Runtime>;
    type Call = Call;
    type Event = Event;
    /// The maximum amount of gas that could be expended per block.
    type BlockGasLimit = dyn Get<Gas>;
    type DetermineContractAddress = contracts::SimpleAddressDeterminator<Runtime>;
    type ComputeDispatchFee = contracts::DefaultDispatchFeeComputor<Runtime>;
    type TrieIdGenerator = contracts::TrieIdFromParentCounter<Runtime>;
    type GasPayment = ();
}

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

construct_runtime!(
    pub enum Runtime where 
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: system::{Module, Call, Storage, Config, Event},
		Timestamp: timestamp::{Module, Call, Storage, Inherent},
		Aura: aura::{Module, Config<T>, Inherent(Timestamp)},
		Grandpa: grandpa::{Module, Call, Storage, Config, Event},
		Indices: indices,
		Balances: balances,
		Sudo: sudo,
		Session: session,
		Staking: staking,
		Democracy: democracy,
		Contract: contracts::{Module, Call, Config, Event},
		Dao: dao::{Module, Call, Storage, Event<T>},
		Marketplace: marketplace::{Module, Call, Storage, Event<T>},
        Token: token::{Module, Call, Storage, Config, Event<T>},
        Bridge: bridge::{Module, Call, Storage, Config, Event<T>},
        RandomnessCollectiveFlip: randomness_collective_flip::{Module, Call, Storage},
	}
);

// The address format for describing accounts.
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
    transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Just that the Signature Signer needs this additional definition as well
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive =
    executive::Executive<Runtime, Block, system::ChainContext<Runtime>, Runtime, AllModules>;

// Implement our runtime API endpoints. This is just a bunch of proxying.
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

	impl block_builder_api::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: inherents::InherentData,
		) -> inherents::CheckInherentsResult {
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

	impl offchain_primitives::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(number: NumberFor<Block>) {
			Executive::offchain_worker(number)
		}
	}

	impl aura_primitives::AuraApi<Block, AuraId> for Runtime {
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
