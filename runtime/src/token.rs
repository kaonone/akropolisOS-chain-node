/// runtime module implementing the ERC20 token interface
use parity_codec::{Codec, Decode, Encode};
use rstd::prelude::*;
use runtime_primitives::traits::{
    As, CheckedAdd, CheckedSub, Member, One, SimpleArithmetic, StaticLookup, Zero,
};
use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, ensure, traits::Currency, Parameter,
    StorageMap, StorageValue,
};
use system::{self, ensure_signed};

// #[derive(Encode, Decode, Default, Clone, PartialEq)]
// #[cfg_attr(feature = "std", derive(Debug))]
// pub struct Token<TokenId> {
//     id: TokenId,
//     symbol: Vec<u8>,
// }

pub trait Trait: balances::Trait + system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type TokenId: Parameter + Member + SimpleArithmetic + Encode + Codec + Default + Copy + As<u64>;
}

decl_storage! {
    trait Store for Module<T: Trait> as TokenStorage {
        Count get(count): T::TokenId;

        // TokenInfo get(token_info): map(T::TokenId) => Token<T::TokenId>;
        TokenId get(token_id): map Vec<u8> => T::TokenId;
        TokenSymbol get(token_symbol): map T::TokenId => Vec<u8>;
        TotalSupply get(total_supply): map T::TokenId => T::Balance;
        Balance get(balance_of): map (T::TokenId, T::AccountId) => T::Balance;
        Allowance get(allowance_of): map (T::TokenId, T::AccountId, T::AccountId) => T::Balance;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;

        fn burn(origin, id: T::TokenId, #[compact] amount: T::Balance) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(Self::total_supply(id) > amount, "Cannot burn more than total supply");

            let old_balance = <Balance<T>>::get((id, sender.clone()));
            let next_balance = old_balance.checked_sub(&amount).ok_or("underflow adding to total supply")?;
            let next_total = Self::total_supply(id).checked_sub(&amount).ok_or("underflow adding to total supply")?;
            <Balance<T>>::insert((id, sender.clone()), next_balance);
            <TotalSupply<T>>::insert(id, next_total);

            Self::deposit_event(RawEvent::Burn(id, sender,  amount));

            Ok(())
        }

        fn mint(origin, exchanger: T::AccountId, #[compact] amount: T::Balance, token: Vec<u8>) {
            let owner = ensure_signed(origin)?;

            ensure!(!amount.is_zero(), "amount should be non-zero");

            let id = if let Some(_) = <TokenId<T>>::exists(&token).into() {
                <TokenId<T>>::get(&token)
                } else {
                    let _ = Self::create_token(owner.clone(), &token)?;
                    Self::count()
                    };

            let next_total = Self::total_supply(id).checked_add(&amount).ok_or("overflow adding to total supply")?;
            <Balance<T>>::insert((id, exchanger.clone()), amount.clone());
            <TotalSupply<T>>::insert(id, next_total);

            <balances::Module<T> as Currency<_>>::deposit_creating(&exchanger, amount);

            Self::deposit_event(RawEvent::Mint(id, owner, amount));
        }

        fn transfer(origin,
            #[compact] id: T::TokenId,
            to: <T::Lookup as StaticLookup>::Source,
            #[compact] amount: T::Balance
        ) {
            let sender = ensure_signed(origin)?;
            let to = T::Lookup::lookup(to)?;
            ensure!(!amount.is_zero(), "transfer amount should be non-zero");

            Self::make_transfer(id, sender, to, amount)?;
        }

        fn approve(origin,
        	#[compact] id: T::TokenId,
        	spender: <T::Lookup as StaticLookup>::Source,
        	#[compact] value: T::Balance
        ) {
        	let sender = ensure_signed(origin)?;
        	let spender = T::Lookup::lookup(spender)?;

        	<Allowance<T>>::insert((id, sender.clone(), spender.clone()), value);

        	Self::deposit_event(RawEvent::Approval(id, sender, spender, value));
        }

        fn transfer_from(origin,
        	#[compact] id: T::TokenId,
        	from: T::AccountId,
        	to: T::AccountId,
        	#[compact] value: T::Balance
        ) {
        	let sender = ensure_signed(origin)?;
        	let allowance = Self::allowance_of((id, from.clone(), sender.clone()));

        	let updated_allowance = allowance.checked_sub(&value).ok_or("underflow in calculating allowance")?;

        	Self::make_transfer(id, from.clone(), to.clone(), value)?;

        	<Allowance<T>>::insert((id, from, sender), updated_allowance);
        }

        // fn deposit(origin, #[compact] token_id: T::TokenId, #[compact] value: T::Balance) {
        // 	let who = ensure_signed(origin)?;
        // 	ensure!(Self::count() > token_id, "Non-existent token");
        //     <balances::Module<T> as Currency<_>>::transfer(&who, token_id, amount)?;
        // 	T::Currency::transfer(&who, &Self::fund_account_id(token_id), value)?;

        // 	Self::deposit_event(RawEvent::Deposit(token_id, who, value));
        // }
    }
}

// events
decl_event!(
    pub enum Event<T>
    where
        TokenId = <T as Trait>::TokenId,
        AccountId = <T as system::Trait>::AccountId,
        Balance = <T as balances::Trait>::Balance,
    {
        NewToken(TokenId, AccountId),
        Transfer(TokenId, AccountId, AccountId, Balance),
        Approval(TokenId, AccountId, AccountId, Balance),
        Mint(TokenId, AccountId, Balance),
        Burn(TokenId, AccountId, Balance),
    }
);

impl<T: Trait> Module<T> {
    fn create_token(owner: T::AccountId, token: &Vec<u8>) -> Result {
        let count = <Count<T>>::get();
        let next_id = count
            .checked_add(&One::one())
            .ok_or("overflow when adding new token")?;

        <Count<T>>::mutate(|n| *n = next_id);
        <TokenSymbol<T>>::insert(next_id, token.clone());

        // let next_token = Token {
        //     id: next_id,    
        //     symbol: token.clone()
        // }
        // <TokenInfo<T>>::insert(next_id, next_token);

        Self::deposit_event(RawEvent::NewToken(<Count<T>>::get(), owner.clone()));

        Ok(())
    }

    pub fn make_transfer(
        id: T::TokenId,
        from: T::AccountId,
        to: T::AccountId,
        amount: T::Balance,
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
