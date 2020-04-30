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
use codec::Encode;
use frame_support::{
    debug, decl_event, decl_module, decl_storage, dispatch, traits::Get,
    weights::SimpleDispatchInfo, IterableStorageMap,
};
#[cfg(not(feature = "std"))]
#[allow(unused)]
use num_traits::float::FloatCore;
use simple_json::{self, json::JsonValue};
use sp_core::crypto::KeyTypeId;
use sp_io::{self, misc::print_utf8 as print_bytes};
use sp_runtime::{
    offchain::http,
    traits::{SaturatedConversion, Zero},
    transaction_validity::{
        InvalidTransaction, TransactionSource, TransactionValidity, ValidTransaction,
    },
};

// We have to import a few things
use sp_std::prelude::*;
use system::ensure_none;
use system::offchain::SubmitUnsignedTransaction;

type Result<T> = core::result::Result<T, &'static str>;

/// Our local KeyType.
///
/// For security reasons the offchain worker doesn't have direct access to the keys
/// but only to app-specific subkeys, which are defined and grouped by their `KeyTypeId`.
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ofpf");

pub const TOKENS_TO_KEEP: usize = 10;

// REVIEW-CHECK: is it necessary to wrap-around storage vector at `MAX_VEC_LEN`?
// pub const MAX_VEC_LEN: usize = 1000;

pub mod crypto {
    pub use super::KEY_TYPE;
    use sp_runtime::app_crypto::{app_crypto, sr25519};
    app_crypto!(sr25519, KEY_TYPE);
}

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
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Call: From<Call<Self>>;
    type SubmitUnsignedTransaction: SubmitUnsignedTransaction<Self, <Self as Trait>::Call>;

    /// A grace period after we send transaction.
    ///
    /// To avoid sending too many transactions, we only attempt to send one
    /// every `GRACE_PERIOD` blocks. We use Local Storage to coordinate
    /// sending between distinct runs of this offchain worker.
    type GracePeriod: Get<Self::BlockNumber>;

    // Wait period between automated fetches. Set to 0 disable this feature.
    //   Then you need to manucally kickoff pricefetch
    type BlockFetchPeriod: Get<Self::BlockNumber>;
}

decl_event!(
    pub enum Event<T>
    where
        Moment = <T as timestamp::Trait>::Moment,
        Balance = <T as balances::Trait>::Balance,
    {
        FetchedPrice(Vec<u8>, Vec<u8>, Moment, Balance),
        AggregatedPrice(Vec<u8>, Moment, Balance),
    }
);

// This module's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as PriceOracle {
    // mapping of token symbol -> (timestamp, price)
    //   price has been inflated by 10,000, and in USD.
    //   When used, it should be divided by 10,000.
    // Using linked map for easy traversal from offchain worker or UI
    pub TokenPriceHistory get(fn token_price_history):
    map hasher(blake2_128_concat) Vec<u8> => Vec<T::Balance>;

    // storage about aggregated price points (calculated with our logic)
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

    #[weight = SimpleDispatchInfo::FixedNormal(10_000)]
    pub fn record_price_unsigned(
        origin,
        _block_number: T::BlockNumber,
        crypto_info: (Vec<u8>, Vec<u8>, Vec<u8>),
        price: T::Balance
    ) -> dispatch::DispatchResult {
        ensure_none(origin)?;

        let (symbol, remote_src) = (crypto_info.0, crypto_info.1);
        let now = <timestamp::Module<T>>::get();

    //     //DEBUG
    //     debug::info!("record_price_unsigned: {:?}, {:?}, {:?}",
    //     core::str::from_utf8(&symbol).map_err(|_| "`symbol` conversion error")?,
    //     core::str::from_utf8(&remote_src).map_err(|_| "`remote_src` conversion error")?,
    //     price
    // );

    <TokenPriceHistory<T>>::mutate(&symbol, |prices| prices.push(price));

      // Spit out an event and Add to storage
      Self::deposit_event(RawEvent::FetchedPrice(symbol, remote_src, now, price));

      Ok(())
    }

    #[weight = SimpleDispatchInfo::FixedNormal(10_000)]
    pub fn record_aggregated_price_points_unsigned(
      origin,
      _block: T::BlockNumber,
      symbol: Vec<u8>,
      price: T::Balance
    ) -> dispatch::DispatchResult {
    //     //DEBUG
    //     debug::info!("record_aggregated_price_points_unsigned: {}: {:?}",
    //     core::str::from_utf8(&symbol).map_err(|_| "`symbol` string conversion error")?,
    //     price
    // );
    ensure_none(origin)?;

    let now = <timestamp::Module<T>>::get();

    let price_pt = (now.clone(), price.clone());
    <AggregatedPrices<T>>::insert(&symbol, price_pt);


    let mut old_vec = <TokenPriceHistory<T>>::get(&symbol);
    let new_vec =  if old_vec.len() < TOKENS_TO_KEEP {
        old_vec
    }else{
        let preserve_from_index = &old_vec.len().checked_sub(TOKENS_TO_KEEP).unwrap_or(9usize);
        old_vec.drain(preserve_from_index..).collect::<Vec<T::Balance>>()
    };
    <TokenPriceHistory<T>>::insert(&symbol, new_vec);

      Self::deposit_event(RawEvent::AggregatedPrice(
        symbol.clone(), now.clone(), price.clone()));

      Ok(())
    }

    fn offchain_worker(block: T::BlockNumber) {
      let duration = T::BlockFetchPeriod::get();

      // Type I task: fetch price
      if duration > 0.into() && block % duration == 0.into() {
        for (symbol, remote_src, remote_url) in FETCHED_CRYPTOS.iter() {
          let _res = Self::fetch_price_unsigned(block, *symbol, *remote_src, *remote_url);

        //   if let Err(e) = res {
        //     debug::error!("Error fetching: {:?}, {:?}: {:?}",
        //     core::str::from_utf8(symbol).unwrap(),
        //     core::str::from_utf8(remote_src).unwrap(),
        //     e);
        //   }
        }
      }

      // Type II task: aggregate price
      <TokenPriceHistory<T>>::iter()
      // filter those to be updated
      .filter(|(_, vec)| vec.len() > 0)
      .for_each(|(symbol, _)| {
        let _res = Self::aggregate_price_points_unsigned(block, &symbol);

        // if let Err(e) = res {
        //   debug::error!("Error aggregating price of {:?}: {:?}",
        //   core::str::from_utf8(&symbol).unwrap(), e);
        // }
        });
    }

  }
}

impl<T: Trait> Module<T> {
    fn fetch_json<'a>(remote_url: &'a [u8]) -> Result<JsonValue> {
        //TODO: add deadline for request
        let remote_url_str = core::str::from_utf8(remote_url)
            .map_err(|_| "Error in converting remote_url to string")?;

        let pending = http::Request::get(remote_url_str)
            .send()
            .map_err(|_| "Error in sending http GET request")?;

        let response = pending
            .wait()
            .map_err(|_| "Error in waiting http response back")?;

        if response.code != 200 {
            debug::warn!("Unexpected status code: {}", response.code);
            return Err("Non-200 status code returned from http request");
        }

        let json_result: Vec<u8> = response.body().collect::<Vec<u8>>();

        // Print out the whole JSON blob
        print_bytes(&json_result);

        let json_val: JsonValue = simple_json::parse_json(
            &core::str::from_utf8(&json_result)
                .map_err(|_| "JSON result cannot convert to string")?,
        )
        .map_err(|_| "JSON parsing error")?;

        Ok(json_val)
    }

    fn fetch_price_unsigned<'a>(
        block: T::BlockNumber,
        symbol: &'a [u8],
        remote_src: &'a [u8],
        remote_url: &'a [u8],
    ) -> Result<()> {
        // //DEBUG
        // debug::info!(
        //     "fetch price unsigned: {:?}:{:?}",
        //     core::str::from_utf8(symbol).unwrap(),
        //     core::str::from_utf8(remote_src).unwrap()
        // );

        let json = Self::fetch_json(remote_url)?;
        let price = match remote_src {
            src if src == b"coingecko" => Self::fetch_price_from_coingecko(json)
                .map_err(|_| "fetch_price_from_coingecko error"),
            src if src == b"cryptocompare" => Self::fetch_price_from_cryptocompare(json)
                .map_err(|_| "fetch_price_from_cryptocompare error"),
            _ => Err("Unknown remote source"),
        }?;

        let call = Call::record_price_unsigned(
            block,
            (symbol.to_vec(), remote_src.to_vec(), remote_url.to_vec()),
            price,
        );

        T::SubmitUnsignedTransaction::submit_unsigned(call)
            .map_err(|_| "fetch_price: submit_unsigned(call) error")?;
        Ok(())
    }


    fn round_value(v: f64) -> T::Balance {
        let mut precisioned: u128 = (v * 1000000000.0).round() as u128;
        precisioned = precisioned * 1000000000; // saturate to 10^18 precision
        let balance = precisioned.saturated_into::<T::Balance>();
        balance
    }

    fn fetch_price_from_cryptocompare(v: JsonValue) -> Result<T::Balance> {
        // Expected JSON shape:
        //   r#"{"USD": 7064.16}"#;
        debug::native::debug!("cryptocompare:{:?}", v.get_object()[0]);
        let val_f64: f64 = v.get_object()[0].1.get_number_f64();
        Ok(Self::round_value(val_f64))
    }

    fn fetch_price_from_coingecko(v: JsonValue) -> Result<T::Balance> {
        // Expected JSON shape:
        //   r#"{"cdai":{"usd": 7064.16}}"#;
        debug::native::debug!("cryptocompare:{:?}", v.get_object()[0]);
        let val_f64: f64 = v.get_object()[0].1.get_object()[0]
            .1
            .get_number_f64();
        Ok(Self::round_value(val_f64))
    }

    fn aggregate_price_points_unsigned<'a>(block: T::BlockNumber, symbol: &'a [u8]) -> Result<()> {
        let token_pricepoints_vec = <TokenPriceHistory<T>>::get(symbol);
        let price_sum: T::Balance = token_pricepoints_vec
            .iter()
            .fold(T::Balance::zero(), |mem, price| mem + *price);

        // Avoiding floating-point arithmetic & do integer division
        let price_avg: T::Balance =
            price_sum / T::Balance::from(token_pricepoints_vec.len() as u32);

        let call = Call::record_aggregated_price_points_unsigned(block, symbol.to_vec(), price_avg);

        T::SubmitUnsignedTransaction::submit_unsigned(call)
            .map_err(|_| "aggregate_price_points: submit_unsigned(call) error")?;

        Ok(())
    }
}

#[allow(deprecated)]
impl<T: Trait> frame_support::unsigned::ValidateUnsigned for Module<T> {
    type Call = Call<T>;

    #[allow(deprecated)]
    fn validate_unsigned(_src: TransactionSource, call: &Self::Call) -> TransactionValidity {
        // debug::info!("Calling {:?}", call);

        match call {
            Call::record_price_unsigned(block, (symbol, ..), price) => Ok(ValidTransaction {
                priority: 1,
                requires: vec![],
                provides: vec![(block, symbol, price).encode()],
                longevity: 5,
                propagate: true,
            }),
            Call::record_aggregated_price_points_unsigned(block, symbol, price) => {
                Ok(ValidTransaction {
                    priority: 1,
                    requires: vec![],
                    provides: vec![(block, symbol, price).encode()],
                    longevity: 5,
                    propagate: true,
                })
            }
            _ => InvalidTransaction::Call.into(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    /// tests for this module
    use super::*;
    use frame_support::{impl_outer_dispatch, impl_outer_origin, parameter_types, weights::Weight};
    use sp_core::H256;
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
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = BlockNumber;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
        type ModuleToIndex = ();
        type AccountData = balances::AccountData<u128>;
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type DbWeight = ();
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

    pub type Extrinsic = TestXt<Call, ()>;
    type SubmitPFTransaction =
        system::offchain::TransactionSubmitter<crypto::Public, Call, Extrinsic>;

    pub type OracleModule = Module<Test>;

    parameter_types! {
        pub const BlockFetchPeriod: BlockNumber = 2;
        pub const GracePeriod: BlockNumber = 5;
    }

    impl Trait for Test {
        type Event = ();
        type Call = Call;
        type SubmitUnsignedTransaction = SubmitPFTransaction;

        // Wait period between automated fetches. Set to 0 disable this feature.
        //   Then you need to manucally kickoff pricefetch
        type GracePeriod = GracePeriod;
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

    #[test]
    fn test_coingecko_parsing() {
        new_test_ext().execute_with(|| {
            let json: Vec<u8> = r#"{"cdai":{"usd": 7064.16}}"#.as_bytes().to_vec();
            let v = simple_json::parse_json(
                &core::str::from_utf8(&json)
                    .map_err(|_| "JSON result cannot convert to string")
                    .expect("failed to parse Vec<u8> to &str"),
            )
            .map_err(|_| "JSON parsing error")
            .expect("failed to parse to JsonValue");
            let result = OracleModule::fetch_price_from_coingecko(v)
                .expect("failed to parse from coingecko");

            assert_eq!(result, 7064160000000000000000);
        });
    }
    #[test]
    fn test_cryptocompare_parsing() {
        new_test_ext().execute_with(|| {
            let json: Vec<u8> = r#"{"USD": 7064.16}"#.as_bytes().to_vec();
            let v = simple_json::parse_json(
                &core::str::from_utf8(&json)
                    .map_err(|_| "JSON result cannot convert to string")
                    .expect("failed to parse Vec<u8> to &str"),
            )
            .map_err(|_| "JSON parsing error")
            .expect("failed to parse to JsonValue");
            let result = OracleModule::fetch_price_from_cryptocompare(v)
                .expect("failed to parse from cryptocompare");

            assert_eq!(result, 7064160000000000000000);
        });
    }
    #[test]
    fn test_simple_parsing() {
        new_test_ext().execute_with(|| {
            let json: Vec<u8> = r#"{"USD": 7064.16}"#.as_bytes().to_vec();
            let v = simple_json::parse_json(
                &core::str::from_utf8(&json)
                    .map_err(|_| "JSON result cannot convert to string")
                    .expect("fail"),
            )
            .map_err(|_| "JSON parsing error")
            .expect("double fail");
            let result = v.get_object()[0].1.get_number_f64();

            assert_eq!(result, 7064.16);
        });
    }
}
