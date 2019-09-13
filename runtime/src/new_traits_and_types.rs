use runtime_primitives::traits::{
    As, MaybeSerializeDebug,  SimpleArithmetic,
};
use support::traits::{Imbalance, ExistenceRequirement, WithdrawReason};
use parity_codec::{Codec, Decode, Encode};
#[cfg(feature = "std")]
use std::fmt;
use rstd::result;

/// Input that adds infinite number of zero after wrapped input.
struct TrailingZeroInput<'a>(&'a [u8]);

/// Descriptive error type
#[cfg(feature = "std")]
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Error(&'static str);

/// Undescriptive error type when compiled for no std
#[cfg(not(feature = "std"))]
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Error;

impl Error {
    #[cfg(feature = "std")]
    /// Error description
    ///
    /// This function returns an actual error str when running in `std`
    /// environment, but `""` on `no_std`.
    pub fn what(&self) -> &'static str {
        self.0
    }

    #[cfg(not(feature = "std"))]
    /// Error description
    ///
    /// This function returns an actual error str when running in `std`
    /// environment, but `""` on `no_std`.
    pub fn what(&self) -> &'static str {
        ""
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn description(&self) -> &str {
        self.0
    }
}

impl From<&'static str> for Error {
    #[cfg(feature = "std")]
    fn from(s: &'static str) -> Error {
        Error(s)
    }

    #[cfg(not(feature = "std"))]
    fn from(_s: &'static str) -> Error {
        Error
    }
}


#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        use std::io::ErrorKind::*;
        match err.kind() {
            NotFound => "io error: NotFound".into(),
            PermissionDenied => "io error: PermissionDenied".into(),
            ConnectionRefused => "io error: ConnectionRefused".into(),
            ConnectionReset => "io error: ConnectionReset".into(),
            ConnectionAborted => "io error: ConnectionAborted".into(),
            NotConnected => "io error: NotConnected".into(),
            AddrInUse => "io error: AddrInUse".into(),
            AddrNotAvailable => "io error: AddrNotAvailable".into(),
            BrokenPipe => "io error: BrokenPipe".into(),
            AlreadyExists => "io error: AlreadyExists".into(),
            WouldBlock => "io error: WouldBlock".into(),
            InvalidInput => "io error: InvalidInput".into(),
            InvalidData => "io error: InvalidData".into(),
            TimedOut => "io error: TimedOut".into(),
            WriteZero => "io error: WriteZero".into(),
            Interrupted => "io error: Interrupted".into(),
            Other => "io error: Other".into(),
            UnexpectedEof => "io error: UnexpectedEof".into(),
            _ => "io error: Unkown".into(),
        }
    }
}

/// Wrapper that implements Input for any `Read` and `Seek` type.
#[cfg(feature = "std")]
pub struct IoReader<R: std::io::Read + std::io::Seek>(pub R);

#[cfg(feature = "std")]
impl<R: std::io::Read + std::io::Seek> Input for IoReader<R> {
    fn remaining_len(&mut self) -> Result<Option<usize>, Error> {
        use std::convert::TryInto;
        use std::io::SeekFrom;

        let old_pos = self.0.seek(SeekFrom::Current(0))?;
        let len = self.0.seek(SeekFrom::End(0))?;

        // Avoid seeking a third time when we were already at the end of the
        // stream. The branch is usually way cheaper than a seek operation.
        if old_pos != len {
            self.0.seek(SeekFrom::Start(old_pos))?;
        }

        len.saturating_sub(old_pos)
            .try_into()
            .map_err(|_| "Input cannot fit into usize length".into())
            .map(Some)
    }

    fn read(&mut self, into: &mut [u8]) -> Result<(), Error> {
        self.0.read_exact(into).map_err(Into::into)
    }
}

impl<'a> Input for TrailingZeroInput<'a> {
    fn remaining_len(&mut self) -> Result<Option<usize>, Error> {
        Ok(None)
    }
    fn read(&mut self, into: &mut [u8]) -> Result<(), Error> {
        let len_from_inner = into.len().min(self.0.len());
        into[..len_from_inner].copy_from_slice(&self.0[..len_from_inner]);
        for i in &mut into[len_from_inner..] {
            *i = 0;
        }
        self.0 = &self.0[len_from_inner..];

        Ok(())
    }
}

//TODO: check why this is even needed to compile
#[cfg(feature = "std")]
impl<'a> std::io::Read for TrailingZeroInput<'a> {
    fn read(&mut self, into: &mut [u8]) -> Result<usize, std::io::Error> {
        let len_from_inner = into.len().min(self.0.len());
        into[..len_from_inner].copy_from_slice(&self.0[..len_from_inner]);
        for i in &mut into[len_from_inner..] {
            *i = 0;
        }
        self.0 = &self.0[len_from_inner..];

        //TODO: if this stays, fix this workaround to be Ok(())
        Ok(0usize)
    }
}

/// Trait that allows reading of data into a slice.
pub trait Input {
	/// Should return the remaining length of the input data. If no information about the input
	/// length is available, `None` should be returned.
	///
	/// The length is used to constrain the preallocation while decoding. Returning a garbage
	/// length can open the doors for a denial of service attack to your application.
	/// Otherwise, returning `None` can decrease the performance of your application.
	fn remaining_len(&mut self) -> Result<Option<usize>, Error>;

	/// Read the exact number of bytes required to fill the given buffer.
	///
	/// Note that this function is similar to `std::io::Read::read_exact` and not
	/// `std::io::Read::read`.
	fn read(&mut self, into: &mut [u8]) -> Result<(), Error>;

	/// Read a single byte from the input.
	fn read_byte(&mut self) -> Result<u8, Error> {
		let mut buf = [0u8];
		self.read(&mut buf[..])?;
		Ok(buf[0])
	}
}

impl<'a> Input for &'a [u8] {
	fn remaining_len(&mut self) -> Result<Option<usize>, Error> {
		Ok(Some(self.len()))
	}

	fn read(&mut self, into: &mut [u8]) -> Result<(), Error> {
		if into.len() > self.len() {
			return Err("Not enough data to fill buffer".into());
		}
		let len = into.len();
		into.copy_from_slice(&self[..len]);
		*self = &self[len..];
		Ok(())
	}
}

/// Provide a simple 4 byte identifier for a type.
pub trait TypeId {
    /// Simple 4 byte identifier.
    const TYPE_ID: [u8; 4];
}

/// A module identifier. These are per module and should be stored in a registry somewhere.
#[derive(Clone, Copy, Eq, PartialEq, Encode, Decode)]
pub struct ModuleId(pub [u8; 8]);

impl TypeId for ModuleId {
    const TYPE_ID: [u8; 4] = *b"modl";
}

/// This type can be converted into and possibly from an AccountId (which itself is generic).
pub trait AccountIdConversion<AccountId>: Sized {
    /// Convert into an account ID. This is infallible.
    fn into_account(&self) -> AccountId { self.into_sub_account(&()) }

    /// Try to convert an account ID into this type. Might not succeed.
    fn try_from_account(a: &AccountId) -> Option<Self> {
        Self::try_from_sub_account::<()>(a).map(|x| x.0)
    }

    /// Convert this value amalgamated with the a secondary "sub" value into an account ID. This is
    /// infallible.
    ///
    /// NOTE: The account IDs from this and from `into_account` are *not* guaranteed to be distinct
    /// for any given value of `self`, nor are different invocations to this with different types
    /// `T`. For example, the following will all encode to the same account ID value:
    /// - `self.into_sub_account(0u32)`
    /// - `self.into_sub_account(vec![0u8; 0])`
    /// - `self.into_account()`
    fn into_sub_account<S: Encode>(&self, sub: S) -> AccountId;

    /// Try to convert an account ID into this type. Might not succeed.
    fn try_from_sub_account<S: Decode>(x: &AccountId) -> Option<(Self, S)>;
}

/// Format is TYPE_ID ++ encode(parachain ID) ++ 00.... where 00... is indefinite trailing zeroes to
/// fill AccountId.
impl<T: Encode + Decode + Default, Id: Encode + Decode + TypeId> AccountIdConversion<T> for Id {
    fn into_sub_account<S: Encode>(&self, sub: S) -> T {
        (Id::TYPE_ID, self, sub).using_encoded(|b|
            T::decode(&mut TrailingZeroInput(b))
        ).unwrap_or_default()
    }

    fn try_from_sub_account<S: Decode>(x: &T) -> Option<(Self, S)> {
        x.using_encoded(|d| {
            if &d[0..4] != Id::TYPE_ID { return None }
            let mut cursor = &d[4..];
            let result = Decode::decode(&mut cursor).or(None)?;
            if cursor.iter().all(|x| *x == 0) {
                Some(result)
            } else {
                None
            }
        })
    }
}



//2.0 fungible Currency
/// Abstraction over a fungible assets system.
pub trait Currency<AccountId> {
	/// The balance of an account.
	type Balance: SimpleArithmetic + As<usize> + As<u64> + Codec + Copy + MaybeSerializeDebug + Default;

	/// The opaque token type for an imbalance. This is returned by unbalanced operations
	/// and must be dealt with. It may be dropped but cannot be cloned.
	type PositiveImbalance: Imbalance<Self::Balance, Opposite=Self::NegativeImbalance>;

	/// The opaque token type for an imbalance. This is returned by unbalanced operations
	/// and must be dealt with. It may be dropped but cannot be cloned.
	type NegativeImbalance: Imbalance<Self::Balance, Opposite=Self::PositiveImbalance>;

	// PUBLIC IMMUTABLES

	/// The combined balance of `who`.
	fn total_balance(who: &AccountId) -> Self::Balance;

	/// Same result as `slash(who, value)` (but without the side-effects) assuming there are no
	/// balance changes in the meantime and only the reserved balance is not taken into account.
	fn can_slash(who: &AccountId, value: Self::Balance) -> bool;

	/// The total amount of issuance in the system.
	fn total_issuance() -> Self::Balance;

	/// The minimum balance any single account may have. This is equivalent to the `Balances` module's
	/// `ExistentialDeposit`.
	fn minimum_balance() -> Self::Balance;

	/// The 'free' balance of a given account.
	///
	/// This is the only balance that matters in terms of most operations on tokens. It alone
	/// is used to determine the balance when in the contract execution environment. When this
	/// balance falls below the value of `ExistentialDeposit`, then the 'current account' is
	/// deleted: specifically `FreeBalance`. Further, the `OnFreeBalanceZero` callback
	/// is invoked, giving a chance to external modules to clean up data associated with
	/// the deleted account.
	///
	/// `system::AccountNonce` is also deleted if `ReservedBalance` is also zero (it also gets
	/// collapsed to zero if it ever becomes less than `ExistentialDeposit`.
	fn free_balance(who: &AccountId) -> Self::Balance;

	/// Returns `Ok` iff the account is able to make a withdrawal of the given amount
	/// for the given reason. Basically, it's just a dry-run of `withdraw`.
	///
	/// `Err(...)` with the reason why not otherwise.
	fn ensure_can_withdraw(
		who: &AccountId,
		_amount: Self::Balance,
		reason: WithdrawReason,
		new_balance: Self::Balance,
	) -> result::Result<(), &'static str>;

	// PUBLIC MUTABLES (DANGEROUS)

	/// Transfer some liquid free balance to another staker.
	///
	/// This is a very high-level function. It will ensure all appropriate fees are paid
	/// and no imbalance in the system remains.
	fn transfer(
		source: &AccountId,
		dest: &AccountId,
		value: Self::Balance,
	) -> result::Result<(), &'static str>;

	/// Deducts up to `value` from the combined balance of `who`, preferring to deduct from the
	/// free balance. This function cannot fail.
	///
	/// The resulting imbalance is the first item of the tuple returned.
	///
	/// As much funds up to `value` will be deducted as possible. If this is less than `value`,
	/// then a non-zero second item will be returned.
	fn slash(
		who: &AccountId,
		value: Self::Balance
	) -> (Self::NegativeImbalance, Self::Balance);

	/// Mints `value` to the free balance of `who`.
	///
	/// If `who` doesn't exist, nothing is done and an Err returned.
	fn deposit_into_existing(
		who: &AccountId,
		value: Self::Balance
	) -> result::Result<Self::PositiveImbalance, &'static str>;

	/// Removes some free balance from `who` account for `reason` if possible. If `liveness` is `KeepAlive`,
	/// then no less than `ExistentialDeposit` must be left remaining.
	///
	/// This checks any locks, vesting, and liquidity requirements. If the removal is not possible, then it
	/// returns `Err`.
	fn withdraw(
		who: &AccountId,
		value: Self::Balance,
		reason: WithdrawReason,
		liveness: ExistenceRequirement,
	) -> result::Result<Self::NegativeImbalance, &'static str>;

	/// Adds up to `value` to the free balance of `who`. If `who` doesn't exist, it is created.
	///
	/// Infallible.
	fn deposit_creating(
		who: &AccountId,
		value: Self::Balance,
	) -> Self::PositiveImbalance;

	/// Ensure an account's free balance equals some value; this will create the account
	/// if needed.
	///
	/// Returns a signed imbalance and status to indicate if the account was successfully updated or update
	/// has led to killing of the account.
	fn make_free_balance_be(
		who: &AccountId,
		balance: Self::Balance,
	) -> (
		SignedImbalance<Self::Balance, Self::PositiveImbalance>,
		UpdateBalanceOutcome,
	);
}
/// Outcome of a balance update.
pub enum UpdateBalanceOutcome {
	/// Account balance was simply updated.
	Updated,
	/// The update led to killing the account.
	AccountKilled,
}

/// Either a positive or a negative imbalance.
pub enum SignedImbalance<B, P: Imbalance<B>>{
	/// A positive imbalance (funds have been created but none destroyed).
	Positive(P),
	/// A negative imbalance (funds have been destroyed but none created).
	Negative(P::Opposite),
}

impl<
	P: Imbalance<B, Opposite=N>,
	N: Imbalance<B, Opposite=P>,
	B: SimpleArithmetic + As<usize> + As<u64> + Codec + Copy + MaybeSerializeDebug + Default,
> SignedImbalance<B, P> {
	pub fn zero() -> Self {
		SignedImbalance::Positive(P::zero())
	}

	pub fn drop_zero(self) -> Result<(), Self> {
		match self {
			SignedImbalance::Positive(x) => x.drop_zero().map_err(SignedImbalance::Positive),
			SignedImbalance::Negative(x) => x.drop_zero().map_err(SignedImbalance::Negative),
		}
	}

	/// Consume `self` and an `other` to return a new instance that combines
	/// both.
	pub fn merge(self, other: Self) -> Self {
		match (self, other) {
			(SignedImbalance::Positive(one), SignedImbalance::Positive(other)) =>
				SignedImbalance::Positive(one.merge(other)),
			(SignedImbalance::Negative(one), SignedImbalance::Negative(other)) =>
				SignedImbalance::Negative(one.merge(other)),
			(SignedImbalance::Positive(one), SignedImbalance::Negative(other)) =>
				if one.peek() > other.peek() {
					SignedImbalance::Positive(one.offset(other).ok().unwrap_or_else(P::zero))
				} else {
					SignedImbalance::Negative(other.offset(one).ok().unwrap_or_else(N::zero))
				},
			(one, other) => other.merge(one),
		}
	}
}