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
use core::convert::From;
use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult,
    traits::Get, weights::SimpleDispatchInfo, IterableStorageMap,
};
#[cfg(not(feature = "std"))]
#[allow(unused)]
use num_traits::float::FloatCore;
use simple_json::{self, json::JsonValue};
use sp_core::crypto::KeyTypeId;
// use sp_io::{self, misc::print_utf8 as print_bytes};
use sp_runtime::{
    offchain::{http, Duration},
    traits::{SaturatedConversion, Zero},
    transaction_validity::{
        InvalidTransaction, TransactionSource, TransactionValidity, ValidTransaction,
    },
};
// We have to import a few things
use sp_std::prelude::*;
use sp_std::str;
use system::ensure_none;
use system::offchain::{SubmitSignedTransaction, SubmitUnsignedTransaction};

/// Our local KeyType.
///
/// For security reasons the offchain worker doesn't have direct access to the keys
/// but only to app-specific subkeys, which are defined and grouped by their `KeyTypeId`.
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ofpf");

pub const TOKENS_TO_KEEP: usize = 10;

pub mod crypto {
    pub use super::KEY_TYPE;
    use sp_runtime::app_crypto::{app_crypto, sr25519};
    app_crypto!(sr25519, KEY_TYPE);
}
pub const HTTP_HEADER_USER_AGENT: &[u8] = b"akropolisos";
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
    /// The type to sign and submit transactions.
    type SubmitSignedTransaction: SubmitSignedTransaction<Self, <Self as Trait>::Call>;
    /// The type to submit unsigned transactions.
    type SubmitUnsignedTransaction: SubmitUnsignedTransaction<Self, <Self as Trait>::Call>;
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
        FetchedPrice(Vec<u8>, Vec<u8>, Moment, Balance),
        AggregatedPrice(Vec<u8>, Moment, Balance),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        SignedCallError,
        UnsignedCallError,
        HttpFetchingError,
        UrlParsingError,
        JsonParsingError,
    }
}

decl_storage! {
  trait Store for Module<T: Trait> as PriceOracle {
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

    #[weight = SimpleDispatchInfo::FixedNormal(10_000)]
    pub fn record_price_signed(
        origin,
        crypto_info: (Vec<u8>, Vec<u8>, Vec<u8>),
        price: T::Balance
    ) -> DispatchResult {
        ensure_none(origin)?;
        Self::_record_price(crypto_info, price)
    }

    #[weight = SimpleDispatchInfo::FixedNormal(10_000)]
    pub fn record_aggregated_prices_signed(
        origin,
        symbol: Vec<u8>,
        price: T::Balance
    ) -> DispatchResult {
        ensure_none(origin)?;
        Self::_record_aggregated_prices(symbol, price)
    }
    #[weight = SimpleDispatchInfo::FixedNormal(10_000)]
    pub fn record_price_unsigned(
        origin,
        _block_number: T::BlockNumber,
        crypto_info: (Vec<u8>, Vec<u8>, Vec<u8>),
        price: T::Balance
    ) -> DispatchResult {
        ensure_none(origin)?;
        Self::_record_price(crypto_info, price)
    }
    #[weight = SimpleDispatchInfo::FixedNormal(10_000)]
    pub fn record_aggregated_prices_unsigned(
      origin,
      _block: T::BlockNumber,
      symbol: Vec<u8>,
      price: T::Balance
    ) -> DispatchResult {
        ensure_none(origin)?;
        Self::_record_aggregated_prices(symbol, price)
    }

    fn offchain_worker(block: T::BlockNumber) {
        debug::RuntimeLogger::init();
        let duration = T::BlockFetchPeriod::get();

        if duration > 0.into() && block % duration == 0.into() {
        let is_even = block % T::BlockNumber::from(2) == T::BlockNumber::from(0);
        // Type I task: fetch price
        for (symbol, remote_src, remote_url) in FETCHED_CRYPTOS.iter() {
          let res = match is_even {
            true =>  Self::fetch_price_signed(*symbol, *remote_src, *remote_url),
            false =>  Self::fetch_price_unsigned(block, *symbol, *remote_src, *remote_url),
          };
          if let Err(e) = res { debug::error!("Error fetching price: {:?}", e); }

        }

        // Type II task: aggregate price
        <TokenPriceHistory<T>>::iter()
        // filter those to be updated
        .filter(|(_, vec)| vec.len() > 0)
        .for_each(|(symbol, _)| {
          let res = match is_even {
              true =>  Self::aggregate_prices_signed(&symbol),
              false =>  Self::aggregate_prices_unsigned(block, &symbol),
            };
            if let Err(e) = res { debug::error!("Error aggregating price: {:?}", e); }
          });
      }
    }
  }
}

impl<T: Trait> Module<T> {
    fn fetch_from_remote<'a>(remote_url: &'a [u8]) -> Result<Vec<u8>, Error<T>> {
        let remote_url_bytes = remote_url.to_vec();
        let user_agent = HTTP_HEADER_USER_AGENT.to_vec();
        let remote_url = str::from_utf8(&remote_url_bytes).map_err(|e| {
            debug::error!("failed to parse remote_url: {:?}", e);
            <Error<T>>::UrlParsingError
        })?;

        debug::info!("sending request to: {}", remote_url);

        let request = http::Request::get(remote_url);

        // Keeping the offchain worker execution time reasonable, so limiting the call to be within 3s.
        let timeout = sp_io::offchain::timestamp().add(Duration::from_millis(3000));

        let pending = request
            .add_header(
                "User-Agent",
                str::from_utf8(&user_agent).map_err(|_| <Error<T>>::HttpFetchingError)?,
            )
            .deadline(timeout) // Setting the timeout time
            .send() // Sending the request out by the host
            .map_err(|_| {
                debug::error!("failed to send request: {:?}", remote_url);
                <Error<T>>::HttpFetchingError
            })?;

        let response = pending
            .try_wait(timeout)
            .map_err(|_| {
                debug::error!("failed to wait for response: {:?}", remote_url);
                <Error<T>>::HttpFetchingError
            })?
            .map_err(|_| {
                debug::error!("failed to unwrap response: {:?}", remote_url);
                <Error<T>>::HttpFetchingError
            })?;

        if response.code != 200 {
            debug::error!("Unexpected http request status code: {}", response.code);
            return Err(<Error<T>>::HttpFetchingError);
        }

        Ok(response.body().collect::<Vec<u8>>())
    }

    fn fetch_json<'a>(remote_url: &'a [u8]) -> Result<JsonValue, Error<T>> {
        let body = Self::fetch_from_remote(remote_url)?;
        // Print out the whole JSON blob
        // print_bytes(&body);

        let json_val: JsonValue = simple_json::parse_json(
            &core::str::from_utf8(&body).map_err(|_| <Error<T>>::JsonParsingError)?,
        )
        .map_err(|_| <Error<T>>::JsonParsingError)?;

        Ok(json_val)
    }

    /*        SIGNED        */
    fn fetch_price_signed<'a>(
        symbol: &'a [u8],
        remote_src: &'a [u8],
        remote_url: &'a [u8],
    ) -> Result<(), Error<T>> {
        // //DEBUG
        // debug::info!(
        //     "fetch price: {:?}:{:?}",
        //     core::str::from_utf8(symbol).unwrap(),
        //     core::str::from_utf8(remote_src).unwrap()
        // );
        let json = Self::fetch_json(remote_url)?;
        let price = match remote_src {
            src if src == b"coingecko" => Self::parse_price_from_coingecko(json),
            src if src == b"cryptocompare" => Self::parse_price_from_cryptocompare(json),
            _ => Err(<Error<T>>::HttpFetchingError),
        }?;

        let call = Call::record_price_signed(
            (symbol.to_vec(), remote_src.to_vec(), remote_url.to_vec()),
            price,
        );

        // Using `SubmitSignedTransaction` associated type we create and submit a transaction
        //   representing the call, we've just created.
        let results = T::SubmitSignedTransaction::submit_signed(call);
        for (acc, res) in &results {
            match res {
                Ok(()) => {
                    debug::native::info!("off-chain record_price: acc: {}", acc,);
                }
                Err(e) => {
                    debug::native::error!("[{:?}] Failed to submit signed tx: {:?}", acc, e);
                    return Err(<Error<T>>::SignedCallError);
                }
            };
        }
        Ok(())
    }

    fn aggregate_prices_signed<'a>(symbol: &'a [u8]) -> Result<(), Error<T>> {
        if !T::SubmitSignedTransaction::can_sign() {
            debug::error!("No local account available");
            return Err(<Error<T>>::SignedCallError);
        }
        let token_pricepoints_vec = <TokenPriceHistory<T>>::get(symbol);
        let price_sum: T::Balance = token_pricepoints_vec
            .iter()
            .fold(T::Balance::zero(), |mem, price| mem + *price);

        // Avoiding floating-point arithmetic & do integer division
        let price_avg: T::Balance =
            price_sum / T::Balance::from(token_pricepoints_vec.len() as u32);

        let call = Call::record_aggregated_prices_signed(symbol.to_vec(), price_avg);

        // Using `SubmitSignedTransaction` associated type we create and submit a transaction
        //   representing the call, we've just created.
        let results = T::SubmitSignedTransaction::submit_signed(call);
        for (acc, res) in &results {
            match res {
                Ok(()) => {
                    debug::native::info!("off-chain record_aggregated_prices: acc: {}", acc,);
                }
                Err(e) => {
                    debug::native::error!("[{:?}] Failed to submit signed tx: {:?}", acc, e);
                    return Err(<Error<T>>::SignedCallError);
                }
            };
        }

        Ok(())
    }

    /*        UNSIGNED          */
    fn fetch_price_unsigned<'a>(
        block: T::BlockNumber,
        symbol: &'a [u8],
        remote_src: &'a [u8],
        remote_url: &'a [u8],
    ) -> Result<(), Error<T>> {
        debug::RuntimeLogger::init();

        // //DEBUG
        // debug::info!(
        //     "fetch price: {:?}:{:?}",
        //     core::str::from_utf8(symbol).unwrap(),
        //     core::str::from_utf8(remote_src).unwrap()
        // );
        let json = Self::fetch_json(remote_url)?;
        let price = match remote_src {
            src if src == b"coingecko" => Self::parse_price_from_coingecko(json),
            src if src == b"cryptocompare" => Self::parse_price_from_cryptocompare(json),
            _ => Err(<Error<T>>::HttpFetchingError),
        }?;

        let call = Call::record_price_unsigned(
            block,
            (symbol.to_vec(), remote_src.to_vec(), remote_url.to_vec()),
            price,
        );

        T::SubmitUnsignedTransaction::submit_unsigned(call).map_err(|e| {
            debug::error!("Failed in fetch_price_unsigned: {:?}", e);
            <Error<T>>::UnsignedCallError
        })
    }
    fn aggregate_prices_unsigned<'a>(
        block: T::BlockNumber,
        symbol: &'a [u8],
    ) -> Result<(), Error<T>> {
        let token_pricepoints_vec = <TokenPriceHistory<T>>::get(symbol);
        let price_sum: T::Balance = token_pricepoints_vec
            .iter()
            .fold(T::Balance::zero(), |mem, price| mem + *price);

        // Avoiding floating-point arithmetic & do integer division
        let price_avg: T::Balance =
            price_sum / T::Balance::from(token_pricepoints_vec.len() as u32);

        let call = Call::record_aggregated_prices_unsigned(block, symbol.to_vec(), price_avg);

        T::SubmitUnsignedTransaction::submit_unsigned(call).map_err(|e| {
            debug::error!("Failed in aggregate_prices_unsigned: {:?}", e);
            <Error<T>>::UnsignedCallError
        })
    }

    fn _record_price(
        crypto_info: (Vec<u8>, Vec<u8>, Vec<u8>),
        price: T::Balance,
    ) -> DispatchResult {
        let (symbol, remote_src) = (crypto_info.0, crypto_info.1);
        let now = <timestamp::Module<T>>::get();

        //     //DEBUG
        //     debug::info!("record_price: {:?}, {:?}, {:?}",
        //     core::str::from_utf8(&symbol).map_err(|_| "`symbol` conversion error")?,
        //     core::str::from_utf8(&remote_src).map_err(|_| "`remote_src` conversion error")?,
        //     price
        // );
        <TokenPriceHistory<T>>::mutate(&symbol, |prices| prices.push(price));

        Self::deposit_event(RawEvent::FetchedPrice(symbol, remote_src, now, price));
        Ok(())
    }
    fn _record_aggregated_prices(symbol: Vec<u8>, price: T::Balance) -> DispatchResult {
        //     //DEBUG
        //     debug::info!("record_aggregated_price_points: {}: {:?}",
        //     core::str::from_utf8(&symbol).map_err(|_| "`symbol` string conversion error")?,
        //     price
        // );

        let now = <timestamp::Module<T>>::get();

        let price_pt = (now.clone(), price.clone());
        <AggregatedPrices<T>>::insert(&symbol, price_pt);

        let mut old_vec = <TokenPriceHistory<T>>::get(&symbol);
        let new_vec = if old_vec.len() < TOKENS_TO_KEEP {
            old_vec
        } else {
            let preserve_from_index = &old_vec.len().checked_sub(TOKENS_TO_KEEP).unwrap_or(9usize);
            old_vec
                .drain(preserve_from_index..)
                .collect::<Vec<T::Balance>>()
        };
        <TokenPriceHistory<T>>::insert(&symbol, new_vec);

        Self::deposit_event(RawEvent::AggregatedPrice(
            symbol.clone(),
            now.clone(),
            price.clone(),
        ));

        Ok(())
    }

    fn round_value(v: f64) -> T::Balance {
        let mut precisioned: u128 = (v * 1000000000.0).round() as u128;
        precisioned = precisioned * 1000000000; // saturate to 10^18 precision
        let balance = precisioned.saturated_into::<T::Balance>();
        balance
    }

    fn parse_price_from_cryptocompare(v: JsonValue) -> Result<T::Balance, Error<T>> {
        // Expected JSON shape:
        //   r#"{"USD": 7064.16}"#;
        debug::info!("cryptocompare:{:?}", v.get_object());
        debug::native::info!("cryptocompare:{:?}", v.get_object());
        let val_f64: f64 = v.get_object()[0].1.get_number_f64();
        Ok(Self::round_value(val_f64))
    }

    fn parse_price_from_coingecko(v: JsonValue) -> Result<T::Balance, Error<T>> {
        // Expected JSON shape:
        //   r#"{"cdai":{"usd": 7064.16}}"#;

        // let val = simple_json::parse_json(price_str);
        // let price = val.ok().and_then(|v| match v {
        // 	JsonValue::Object(obj) => {
        // 		let mut chars = "USD".chars();
        // 		obj.into_iter()
        // 			.find(|(k, _)| k.iter().all(|k| Some(*k) == chars.next()))
        // 			.and_then(|v| match v.1 {
        // 				JsonValue::Number(number) => Some(number),
        // 				_ => None,
        // 			})
        // 	},
        // 	_ => None
        // })?;
        debug::info!("cryptocompare:{:?}", v.get_object());
        debug::native::info!("cryptocompare:{:?}", v.get_object());
        let val_f64: f64 = v.get_object()[0].1.get_object()[0].1.get_number_f64();
        Ok(Self::round_value(val_f64))
    }
}

impl<T: Trait> frame_support::unsigned::ValidateUnsigned for Module<T> {
    type Call = Call<T>;

    fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
        match call {
            Call::record_price_unsigned(block, (symbol, ..), price) => Ok(ValidTransaction {
                priority: 1 << 20,
                requires: vec![],
                provides: vec![(block, symbol, price).encode()],
                longevity: 5,
                propagate: false,
            }),
            Call::record_aggregated_prices_unsigned(block, symbol, price) => Ok(ValidTransaction {
                priority: 1 << 20,
                requires: vec![],
                provides: vec![(block, symbol, price).encode()],
                longevity: 5,
                propagate: false,
            }),
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
    }

    impl Trait for Test {
        type Event = ();
        type Call = Call;
        type SubmiTransaction = SubmitPFTransaction;
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
            let result = OracleModule::parse_price_from_coingecko(v)
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
            let result = OracleModule::parse_price_from_cryptocompare(v)
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
