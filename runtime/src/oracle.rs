/// Pallet for realworld price oracle requests.
///
/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs
///
/// This module based on project written by Jimmy Chu
/// https://github.com/jimmychu0807/substrate-offchain-pricefetch
///
/// and alpha release example-offchain-worker frame
/// https://github.com/paritytech/substrate/blob/master/frame/example-offchain-worker/src/lib.rs
///
use core::convert::From;
use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, traits::Get,
};
#[cfg(not(feature = "std"))]
#[allow(unused)]
use num_traits::float::FloatCore;
// use sp_io::{self, misc::print_utf8 as print_bytes};
use sp_runtime::traits::Zero;
// We have to import a few things
use sp_std::prelude::*;
use sp_std::str;
use system::ensure_none;

pub const TOKENS_TO_KEEP: usize = 10;

pub const FETCHED_CRYPTOS: [(&[u8], &[u8], &[u8]); 4] = [
    (
        b"DAI",
        b"cryptocompare",
        b"https://min-api.cryptocompare.com/data/price?fsym=DAI&tsyms=USD",
    ),
    (
        b"USDT",
        b"cryptocompare",
        b"https://min-api.cryptocompare.com/data/price?fsym=USDT&tsyms=USD",
    ),
    (
        b"USDC",
        b"cryptocompare",
        b"https://min-api.cryptocompare.com/data/price?fsym=USDC&tsyms=USD",
    ),
    (
        b"cDAI",
        b"coingecko",
        b"https://api.coingecko.com/api/v3/simple/price?ids=cDAI&vs_currencies=USD",
    ),
];

/// The module's configuration trait.
pub trait Trait: timestamp::Trait + balances::Trait + system::Trait {
    /// The overarching dispatch call type.
    type Call: From<Call<Self>>;
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// Wait period between automated fetches. Set to 0 disable this feature.
    /// Then you need to manucally kickoff pricefetch
    type BlockFetchPeriod: Get<Self::BlockNumber>;
}

decl_event!(
    pub enum Event<T>
    where
        Moment = <T as timestamp::Trait>::Moment,
        Balance = <T as balances::Trait>::Balance,
    {
        RecordedPrice(Vec<u8>, Moment, Balance),
        AggregatedPrice(Vec<u8>, Moment, Balance),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        SignedCallError,
        UnsignedCallError,
        HttpFetchingError,
        UrlParsingError,
        AggregatingError,
    }
}

decl_storage! {
  trait Store for Module<T: Trait> as Oracle {
    /// List of last prices with length of TOKENS_TO_KEEP
    pub TokenPriceHistory get(fn token_price_history):
    map hasher(blake2_128_concat) Vec<u8> => Vec<T::Balance>;

    /// Tuple of timestamp and average price for token
    pub AggregatedPrices get(fn aggregated_prices):
    map hasher(blake2_128_concat) Vec<u8> => (T::Moment, T::Balance);
  }
}

// The module's dispatchable functions.
decl_module! {
  /// The module declaration.
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    // Initializing events
    // this is needed only if you are using events in your module
    fn deposit_event() = default;

    #[weight = 0]
    pub fn record_price(
        origin,
        sym: Vec<u8>,
        price: T::Balance
    ) -> DispatchResult {
        debug::RuntimeLogger::init();
        ensure_none(origin)?;
        Self::_record_price(sym, price)
    }

    #[weight = 0]
    pub fn record_aggregated_prices(
        origin,
    ) -> DispatchResult {
        debug::RuntimeLogger::init();
        ensure_none(origin)?;

        Self::_record_aggregated_prices()
    }
    fn on_finalize(_n : T::BlockNumber){
        let block = <system::Module<T>>::block_number();
        if block % T::BlockFetchPeriod::get() == T::BlockNumber::from(0) {
            debug::info!("run aggregate prices :{:?}", block);
            let _ = Self::_record_aggregated_prices();
        }
    }
  }
}

impl<T: Trait> Module<T> {
    fn aggregate_prices<'a>(symbol: &'a [u8]) -> T::Balance {
        let token_pricepoints_vec = <TokenPriceHistory<T>>::get(symbol);
        let price_sum: T::Balance = token_pricepoints_vec
            .iter()
            .fold(T::Balance::zero(), |mem, price| mem + *price);

        match token_pricepoints_vec.len() {
            0 => T::Balance::from(0),
            _ => price_sum / T::Balance::from(token_pricepoints_vec.len() as u32),
        }
    }

    fn _record_price(symbol: Vec<u8>, price: T::Balance) -> DispatchResult {
        let now = <timestamp::Module<T>>::get();

        //     //DEBUG
        //     debug::info!("record_price: {:?}, {:?}, {:?}",
        //     core::str::from_utf8(&symbol).map_err(|_| "`symbol` conversion error")?,
        //     core::str::from_utf8(&remote_src).map_err(|_| "`remote_src` conversion error")?,
        //     price
        // );
        <TokenPriceHistory<T>>::mutate(&symbol, |prices| prices.push(price));

        Self::deposit_event(RawEvent::RecordedPrice(symbol, now, price));
        Ok(())
    }
    fn _record_aggregated_prices() -> DispatchResult {
        //     //DEBUG
        //     debug::info!("record_aggregated_price_points: {}: {:?}",
        //     core::str::from_utf8(&symbol).map_err(|_| "`symbol` string conversion error")?,
        //     price
        // );
        let result = FETCHED_CRYPTOS
            .iter()
            .map(|t| {
                let symbol = t.0;
                let mut old_vec = <TokenPriceHistory<T>>::get(symbol);
                if old_vec.len() == 0 {
                    return Err(<Error<T>>::AggregatingError);
                }
                let price = Self::aggregate_prices(symbol);
                let now = <timestamp::Module<T>>::get();
                let price_pt = (now.clone(), price.clone());
                <AggregatedPrices<T>>::insert(symbol, price_pt);

                let new_vec = if old_vec.len() < TOKENS_TO_KEEP {
                    old_vec
                } else {
                    let preserve_from_index =
                        &old_vec.len().checked_sub(TOKENS_TO_KEEP).unwrap_or(9usize);
                    old_vec
                        .drain(preserve_from_index..)
                        .collect::<Vec<T::Balance>>()
                };
                <TokenPriceHistory<T>>::insert(symbol, new_vec);

                Self::deposit_event(RawEvent::AggregatedPrice(
                    symbol.clone().to_vec(),
                    now.clone(),
                    price.clone(),
                ));
                Ok(())
            })
            .fold(
                Err(<Error<T>>::AggregatingError),
                |_, el: Result<(), Error<T>>| match el {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                },
            );

        match result.is_ok() {
            true => debug::info!("Aggregating prices is successful"),
            false => debug::error!("Error aggregating prices. Check the price lists!"),
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    /// tests for this module
    use super::*;
    use frame_support::{
        impl_outer_dispatch, impl_outer_origin, parameter_types,
        weights::{
            Weight,
            constants::{
                BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND,
            },
        },
    };
    use sp_core::{sr25519, H256};
    use sp_runtime::{
        testing::{Header, TestXt},
        traits::{BlakeTwo256, IdentityLookup},
        Perbill,
    };
    use std::cell::RefCell;

    pub type Balance = u128;
    pub type BlockNumber = u64;

    thread_local! {
        static EXISTENTIAL_DEPOSIT: RefCell<u128> = RefCell::new(500);
    }

    impl_outer_origin! {
      pub enum Origin for Test {}
    }

    impl_outer_dispatch! {
      pub enum Call for Test where origin: Origin {
        price_fetch::OracleModule,
      }
    }

    pub struct ExistentialDeposit;
    impl Get<u128> for ExistentialDeposit {
        fn get() -> u128 {
            EXISTENTIAL_DEPOSIT.with(|v| *v.borrow())
        }
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;

    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }
    impl system::Trait for Test {
        type AccountId = u64;
        type Call = Call;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Index = u64;
        type BlockNumber = BlockNumber;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Header = Header;
        type Event = ();
        type Origin = Origin;
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type BlockExecutionWeight = BlockExecutionWeight;
        type DbWeight = RocksDbWeight;
        type ExtrinsicBaseWeight = ExtrinsicBaseWeight;
        type Version = ();
        type ModuleToIndex = ();
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type AccountData = balances::AccountData<Balance>;
    }

    impl balances::Trait for Test {
        type Balance = Balance;
        type DustRemoval = ();
        type Event = ();
        type ExistentialDeposit = ExistentialDeposit;
        type AccountStore = system::Module<Test>;
    }

    impl timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = ();
    }

    pub type OracleModule = Module<Test>;

    parameter_types! {
        pub const BlockFetchPeriod: BlockNumber = 2;
    }

    impl Trait for Test {
        type Event = ();
        type Call = Call;
        type BlockFetchPeriod = BlockFetchPeriod;
    }

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    pub fn new_test_ext() -> sp_io::TestExternalities {
        system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap()
            .into()
    }
}
