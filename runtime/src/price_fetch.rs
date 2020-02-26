// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use frame_support::{debug, decl_event, decl_module, decl_storage, dispatch, traits::Get};
#[cfg(not(feature = "std"))]
use num_traits::float::FloatCore;
use simple_json::{self, json::JsonValue};
use sp_core::crypto::KeyTypeId;
use sp_io::{self, misc::print_utf8 as print_bytes};
use sp_runtime::{
    offchain::http,
    transaction_validity::{
        InvalidTransaction, TransactionLongevity, TransactionValidity, ValidTransaction,
    },
};
/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs
// This module based on project written by Jimmy Chu
// https://github.com/jimmychu0807/substrate-offchain-pricefetch
// and alpha release example-offchain-worker frame
// https://github.com/paritytech/substrate/blob/master/frame/example-offchain-worker/src/lib.rs

// Heavily depends on substrate commit implementing crucial offchain worker functionality
// https://github.com/paritytech/substrate/commit/8974349874588de655b7350737bba45032bb2548#diff-7f920cccb57d91272a863f03572e5dee

// We have to import a few things
use sp_std::prelude::*;
use system::ensure_none;
use system::offchain::{SubmitSignedTransaction, SubmitUnsignedTransaction};

type Result<T> = core::result::Result<T, &'static str>;

/// Our local KeyType.
///
/// For security reasons the offchain worker doesn't have direct access to the keys
/// but only to app-specific subkeys, which are defined and grouped by their `KeyTypeId`.
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ofpf");

// REVIEW-CHECK: is it necessary to wrap-around storage vector at `MAX_VEC_LEN`?
pub const MAX_VEC_LEN: usize = 1000;

pub mod crypto {
    pub use super::KEY_TYPE;
    use sp_runtime::app_crypto::{app_crypto, sr25519};
    app_crypto!(sr25519, KEY_TYPE);
}

pub const FETCHED_CRYPTOS: [(&[u8], &[u8], &[u8]); 2] = [
    // pub const FETCHED_CRYPTOS: [(&[u8], &[u8], &[u8]); 6] = [
    //   (b"BTC", b"coincap",
    //     b"https://api.coincap.io/v2/assets/bitcoin"),
    //   (b"BTC", b"cryptocompare",
    //     b"https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD"),
    //   (b"ETH", b"coincap",
    //    b"https://api.coincap.io/v2/assets/ethereum"),
    //   (b"ETH", b"cryptocompare",
    // b"https://min-api.cryptocompare.com/data/price?fsym=ETH&tsyms=USD"),
    (b"DAI", b"coincap", b"https://api.coincap.io/v2/assets/dai"),
    (
        b"DAI",
        b"cryptocompare",
        b"https://min-api.cryptocompare.com/data/price?fsym=DAI&tsyms=USD",
    ),
];

/// The module's configuration trait.
pub trait Trait: timestamp::Trait + system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Call: From<Call<Self>>;
    type SubmitUnsignedTransaction: SubmitUnsignedTransaction<Self, <Self as Trait>::Call>;
    type SubmitSignedTransaction: SubmitSignedTransaction<Self, <Self as Trait>::Call>;

    // Wait period between automated fetches. Set to 0 disable this feature.
    //   Then you need to manucally kickoff pricefetch
    type BlockFetchPeriod: Get<Self::BlockNumber>;
}

decl_event!(
    pub enum Event<T>
    where
        Moment = <T as timestamp::Trait>::Moment,
    {
        FetchedPrice(Vec<u8>, Vec<u8>, Moment, u64),
        AggregatedPrice(Vec<u8>, Moment, u64),
    }
);

// This module's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as PriceFetch {
    // mapping of token symbol -> (timestamp, price)
    //   price has been inflated by 10,000, and in USD.
    //   When used, it should be divided by 10,000.
    // Using linked map for easy traversal from offchain worker or UI
    TokenPriceHistory: linked_map Vec<u8> => Vec<(T::Moment, u64)>;

    // storage about aggregated price points (calculated with our logic)
    AggregatedPrices: linked_map Vec<u8> => (T::Moment, u64);
  }
}

// The module's dispatchable functions.
decl_module! {
  /// The module declaration.
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    // Initializing events
    // this is needed only if you are using events in your module
    fn deposit_event() = default;

    pub fn record_price(
      origin,
      _block: T::BlockNumber,
      crypto_info: (Vec<u8>, Vec<u8>, Vec<u8>),
      price: u64
    ) -> dispatch::DispatchResult {
      // Ensuring this is an unsigned tx
      ensure_none(origin)?;

      let (symbol, remote_src) = (crypto_info.0, crypto_info.1);
      let now = <timestamp::Module<T>>::get();

      // Debug printout
      debug::info!("record_price: {:?}, {:?}, {:?}",
        core::str::from_utf8(&symbol).map_err(|_| "`symbol` conversion error")?,
        core::str::from_utf8(&remote_src).map_err(|_| "`remote_src` conversion error")?,
        price
      );

      <TokenPriceHistory<T>>::mutate(&symbol, |pp_vec| pp_vec.push((now, price)));

      // Spit out an event and Add to storage
      Self::deposit_event(RawEvent::FetchedPrice(symbol, remote_src, now, price));

      Ok(())
    }

    pub fn record_aggregated_price_points(
      origin,
      _block: T::BlockNumber,
      symbol: Vec<u8>,
      price: u64
    ) -> dispatch::DispatchResult {
      // Debug printout
      debug::info!("record_aggregated_price_points: {}: {:?}",
        core::str::from_utf8(&symbol).map_err(|_| "`symbol` string conversion error")?,
        price
      );

      // Ensuring this is an unsigned tx
      ensure_none(origin)?;

      let now = <timestamp::Module<T>>::get();

      // Record in the storage
      let price_pt = (now.clone(), price.clone());
      <AggregatedPrices<T>>::insert(&symbol, price_pt);

      // Remove relevant storage items
      <TokenPriceHistory<T>>::remove(&symbol);

      // Spit the event
      Self::deposit_event(RawEvent::AggregatedPrice(
        symbol.clone(), now.clone(), price.clone()));

      Ok(())
    }

    fn offchain_worker(block: T::BlockNumber) {
      let duration = T::BlockFetchPeriod::get();

      // Type I task: fetch price
      if duration > 0.into() && block % duration == 0.into() {
        for (symbol, remote_src, remote_url) in FETCHED_CRYPTOS.iter() {
          if let Err(e) = Self::fetch_price(block, *symbol, *remote_src, *remote_url) {
            debug::error!("Error fetching: {:?}, {:?}: {:?}",
              core::str::from_utf8(symbol).unwrap(),
              core::str::from_utf8(remote_src).unwrap(),
              e);
          }
        }
      }

      // Type II task: aggregate price
      <TokenPriceHistory<T>>::enumerate()
        // filter those to be updated
        .filter(|(_, vec)| vec.len() > 0)
        .for_each(|(symbol, _)| {
          if let Err(e) = Self::aggregate_price_points(block, &symbol) {
            debug::error!("Error aggregating price of {:?}: {:?}",
              core::str::from_utf8(&symbol).unwrap(), e);
          }
        });
    } // end of `fn offchain_worker()`

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

    fn fetch_price<'a>(
        block: T::BlockNumber,
        symbol: &'a [u8],
        remote_src: &'a [u8],
        remote_url: &'a [u8],
    ) -> Result<()> {
        debug::info!(
            "fetch price: {:?}:{:?}",
            core::str::from_utf8(symbol).unwrap(),
            core::str::from_utf8(remote_src).unwrap()
        );

        let json = Self::fetch_json(remote_url)?;

        let price = match remote_src {
            src if src == b"coincap" => {
                Self::fetch_price_from_coincap(json).map_err(|_| "fetch_price_from_coincap error")
            }
            src if src == b"cryptocompare" => Self::fetch_price_from_cryptocompare(json)
                .map_err(|_| "fetch_price_from_cryptocompare error"),
            _ => Err("Unknown remote source"),
        }?;

        let call = Call::record_price(
            block,
            (symbol.to_vec(), remote_src.to_vec(), remote_url.to_vec()),
            price,
        );

        if !T::SubmitSignedTransaction::can_sign() {
            return Err(
                "No local accounts available. Consider adding one via `author_insertKey` RPC.",
            )?;
        } else {
            // Unsigned tx
            // T::SubmitUnsignedTransaction::submit_unsigned(call)
            //     .map_err(|_| "fetch_price: submit_unsigned(call) error");

            // Using `SubmitSignedTransaction` associated type we create and submit a transaction
            // representing the call, we've just created.
            // Submit signed will return a vector of results for all accounts that were found in the
            // local keystore with expected `KEY_TYPE`.
            let results = T::SubmitSignedTransaction::submit_signed(call);
            for (acc, res) in &results {
                match res {
                    Ok(()) => debug::info!("[{:?}] Submitted price of {} cents", acc, price),
                    Err(e) => debug::error!("[{:?}] Failed to submit transaction: {:?}", acc, e),
                }
            }
        }
        Ok(())
    }

    fn vecchars_to_vecbytes<I: IntoIterator<Item = char> + Clone>(it: &I) -> Vec<u8> {
        it.clone().into_iter().map(|c| c as u8).collect::<_>()
    }

    fn fetch_price_from_cryptocompare(json_val: JsonValue) -> Result<u64> {
        // Expected JSON shape:
        //   r#"{"USD": 7064.16}"#;
        let val_f64: f64 = json_val.get_object()[0].1.get_number_f64();
        Ok((val_f64 * 10000.).round() as u64)
    }

    fn fetch_price_from_coincap(json_val: JsonValue) -> Result<u64> {
        // Expected JSON shape:
        //   r#"{"data":{"priceUsd":"8172.2628346190447316"}}"#;

        const PRICE_KEY: &[u8] = b"priceUsd";
        let data = json_val.get_object()[0].1.get_object();

        let (_, v) = data
            .iter()
            .filter(|(k, _)| PRICE_KEY.to_vec() == Self::vecchars_to_vecbytes(k))
            .nth(0)
            .ok_or("fetch_price_from_coincap: JSON does not conform to expectation")?;

        // `val` contains the price, such as "222.333" in bytes form
        let val_u8: Vec<u8> = v.get_bytes();

        // Convert to number
        let val_f64: f64 = core::str::from_utf8(&val_u8)
            .map_err(|_| "fetch_price_from_coincap: val_f64 convert to string error")?
            .parse::<f64>()
            .map_err(|_| "fetch_price_from_coincap: val_u8 parsing to f64 error")?;
        Ok((val_f64 * 10000.).round() as u64)
    }

    fn aggregate_price_points<'a>(block: T::BlockNumber, symbol: &'a [u8]) -> Result<()> {
        let token_pricepoints_vec = <TokenPriceHistory<T>>::get(symbol);
        let price_sum: u64 = token_pricepoints_vec.iter().fold(0, |mem, pp| mem + pp.1);

        // Avoiding floating-point arithmetic & do integer division
        let price_avg: u64 = price_sum / (token_pricepoints_vec.len() as u64);

        // submit onchain call for aggregating the price
        let call = Call::record_aggregated_price_points(block, symbol.to_vec(), price_avg);

        // Unsigned tx
        // T::SubmitUnsignedTransaction::submit_unsigned(call)
        //     .map_err(|_| "aggregate_price_points: submit_signed(call) error");

        // Using `SubmitSignedTransaction` associated type we create and submit a transaction
        // representing the call, we've just created.
        // Submit signed will return a vector of results for all accounts that were found in the
        // local keystore with expected `KEY_TYPE`.
        let results = T::SubmitSignedTransaction::submit_signed(call);
        for (acc, res) in &results {
            match res {
                Ok(()) => debug::info!("[{:?}] Submitted price of {} cents", acc, price_avg),
                Err(e) => debug::error!("[{:?}] Failed to submit transaction: {:?}", acc, e),
            }
        }

        Ok(())
    }
}

#[allow(deprecated)]
impl<T: Trait> frame_support::unsigned::ValidateUnsigned for Module<T> {
    type Call = Call<T>;

    #[allow(deprecated)]
    fn validate_unsigned(call: &Self::Call) -> TransactionValidity {
        match call {
            Call::record_price(block, (symbol, remote_src, ..), price) => Ok(ValidTransaction {
                priority: 0,
                requires: vec![],
                provides: vec![(block, symbol, remote_src, price).encode()],
                longevity: TransactionLongevity::max_value(),
                propagate: true,
            }),
            Call::record_aggregated_price_points(block, symbol, price) => Ok(ValidTransaction {
                priority: 0,
                requires: vec![],
                provides: vec![(block, symbol, price).encode()],
                longevity: TransactionLongevity::max_value(),
                propagate: true,
            }),
            _ => InvalidTransaction::Call.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    /// tests for this module
    // Test cases:
    //  1. record_price if called store item in storage
    //  2. record_price can only be called from unsigned tx
    //  3. with multiple record_price of same symbol inserted. On next cycle, the average of the price is calculated
    //  4. can fetch for BTC, parse the JSON blob and get a price > 0 out
    use super::*;
    use primitives::H256;
    use sp_runtime::{
        testing::{Header, TestXt},
        traits::{BlakeTwo256, IdentityLookup},
        Perbill,
    };
    use support::{impl_outer_dispatch, impl_outer_origin, parameter_types, weights::Weight};

    impl_outer_origin! {
      pub enum Origin for TestRuntime {}
    }

    impl_outer_dispatch! {
      pub enum Call for TestRuntime where origin: Origin {
        price_fetch::PriceFetchModule,
      }
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct TestRuntime;

    parameter_types! {
      pub const BlockHashCount: u64 = 250;
      pub const MaximumBlockWeight: Weight = 1024;
      pub const MaximumBlockLength: u32 = 2 * 1024;
      pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }

    impl system::Trait for TestRuntime {
        type Origin = Origin;
        type Call = Call;
        type Index = u64;
        type BlockNumber = u64;
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
    }

    impl timestamp::Trait for TestRuntime {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = ();
    }

    pub type Extrinsic = TestXt<Call, ()>;
    type SubmitPFTransaction = system::offchain::TransactionSubmitter<(), Call, Extrinsic>;

    parameter_types! {
      pub const BlockFetchPeriod: u64 = 1;
    }

    pub type PriceFetchModule = Module<TestRuntime>;

    impl Trait for TestRuntime {
        type Event = ();
        type Call = Call;
        type SubmitUnsignedTransaction = SubmitPFTransaction;

        // Wait period between automated fetches. Set to 0 disable this feature.
        //   Then you need to manucally kickoff pricefetch
        type BlockFetchPeriod = BlockFetchPeriod;
    }

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    pub fn new_test_ext() -> runtime_io::TestExternalities {
        system::GenesisConfig::default()
            .build_storage::<TestRuntime>()
            .unwrap()
            .into()
    }

    #[test]
    fn it_works_for_default_value() {
        new_test_ext().execute_with(|| {
            assert_eq!(1, 1);
        });
    }
}
