/// runtime module implementing the ERC20 token interface
use rstd::prelude::*;
use runtime_primitives::traits::{
    As, CheckedAdd, CheckedSub, Member, One, SimpleArithmetic, 
};
#[cfg(feature = "std")]
use std::borrow::Borrow;
#[cfg(feature = "std")]
use std::fmt::Debug;
use support::{
    decl_event, decl_module, decl_storage,
    dispatch::Result,
    ensure,
    traits::{Currency, ExistenceRequirement, WithdrawReason},
    Parameter, StorageMap, StorageValue,
};
use system::{self, ensure_signed};
use crate::new_traits_and_types::{AccountIdConversion, ModuleId};
use parity_codec::{Codec, Encode};

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

const MODULE_ID: ModuleId = ModuleId(*b"coinfund");

// use new_types;
pub trait Trait: balances::Trait + system::Trait {
    type Currency: Currency<Self::AccountId>;

    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type TokenBalance: Parameter
        + Member
        + SimpleArithmetic
        + Codec
        + Default
        + Copy
        + As<usize>
        + As<u64>;

    type TokenId: Parameter + Member + SimpleArithmetic + Encode + Codec + Default + Copy + Debug + As<u64>;
}

decl_storage! {
    trait Store for Module<T: Trait> as TokenFactory {
        Init get(is_init): map T::TokenId => bool;
        Count get(count): T::TokenId;

        TotalSupply get(total_supply): map T::TokenId => T::TokenBalance;
        Balance get(balance_of): map (T::TokenId, T::AccountId) => T::TokenBalance;
        Allowance get(allowance): map (T::TokenId, T::AccountId, T::AccountId) => T::TokenBalance;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;

        pub fn burn(origin, id: T::TokenId, #[compact] value: T::TokenBalance) -> Result {
            let sender = ensure_signed(origin)?;

            Self::deposit_event(RawEvent::Burn(id, sender,  value));

            Ok(())
        }

        pub fn issue(origin, #[compact] total_supply: T::TokenBalance, deposit: BalanceOf<T>) {
            let owner = ensure_signed(origin)?;

            let id = Self::count();
            let next_id = id.checked_add(&One::one()).ok_or("overflow when adding new token")?;
            let imbalance = T::Currency::withdraw(&owner, deposit, WithdrawReason::Transfer, ExistenceRequirement::KeepAlive)?;

            <Balance<T>>::insert((id, owner.clone()), total_supply);
            <TotalSupply<T>>::insert(id, total_supply);
            <Count<T>>::put(next_id);

            // T::Currency::resolve_creating(&Self::fund_account_id(id), imbalance);

            Self::deposit_event(RawEvent::NewToken(id, owner.clone(), total_supply));

            Self::deposit_event(RawEvent::Deposit(id, owner, deposit));
        }
    }
}

// events
decl_event!(
    pub enum Event<T>
    where
        TokenId = <T as Trait>::TokenId,
        AccountId = <T as system::Trait>::AccountId,
        TokenBalance = <T as Trait>::TokenBalance,
        Balance = BalanceOf<T>,
    {
        NewToken(TokenId, AccountId, TokenBalance),
        Transfer(TokenId, AccountId, AccountId, TokenBalance),
        Approval(TokenId, AccountId, AccountId, TokenBalance),
        Deposit(TokenId, AccountId, Balance),
        Burn(TokenId, AccountId, TokenBalance),
    }
);

impl<T: Trait> Module<T>
where
    <T as system::Trait>::AccountId:
        Borrow<(<T as Trait>::TokenId, <T as system::Trait>::AccountId)>,
{
    fn init(owner: T::AccountId, id: T::TokenId) -> Result {
        ensure!(Self::is_init(id) == false, "Token already initialized.");

        <Balance<T>>::insert(owner, Self::total_supply(id));
        <Init<T>>::insert(id, true);

        Ok(())
    }

    pub fn fund_account_id(index: T::TokenId) -> T::AccountId {
        MODULE_ID.into_sub_account(index)
    }

    pub fn make_transfer(
        id: T::TokenId,
        from: T::AccountId,
        to: T::AccountId,
        amount: T::TokenBalance,
    ) -> Result {
        let from_balance = Self::balance_of((id, from.clone()));
        ensure!(
            from_balance >= amount.clone(),
            "user does not have enough tokens"
        );

        <Balance<T>>::insert((id, from.clone()), from_balance - amount.clone());
        <Balance<T>>::mutate((id, to.clone()), |balance| *balance += amount.clone());

        Self::deposit_event(RawEvent::Transfer(id, from, to, amount));

        Ok(())
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use primitives::{Blake2Hasher, H256};
    use runtime_io::with_externalities;
    use runtime_primitives::{
        testing::{Digest, DigestItem, Header},
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    };
    use support::{assert_noop, assert_ok, impl_outer_origin};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    impl system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Digest = Digest;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type Log = DigestItem;
    }
    impl balances::Trait for Test {
        type Balance = u128;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type TransactionPayment = ();
        type TransferPayment = ();
        type DustRemoval = ();
        type Event = ();
    }
    impl timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
    }
    impl Trait for Test {
        type Event = ();
    }
    type TokenModule = Module<Test>;

    const TOKEN_NAME: &[u8; 10] = b"NAME";
    const TOKEN_DESC: &[u8; 10] = b"Description-1234_";
    const USER1: u64 = 1;
    const USER2: u64 = 2;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap()
            .0
            .into()
    }

    #[test]
    fn create_token_init() {
        with_externalities(&mut new_test_ext(), || {
            // assert_eq!();
        })
    }
}
