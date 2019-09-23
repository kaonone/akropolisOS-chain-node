/// runtime module implementing the ERC20 token factory API
/// You can use mint to create tokens or burn created tokens
/// and transfer tokens on substrate side freely or operate with total_supply
///
use crate::types::{TokenBalance, TokenId};
use parity_codec::{Decode, Encode};
use rstd::prelude::Vec;
use runtime_primitives::traits::{One, StaticLookup, Zero};
use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, ensure, StorageMap, StorageValue,
};
use system::{self, ensure_signed};

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Token {
    pub id: TokenId,
    pub symbol: Vec<u8>,
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        NewToken(TokenId, AccountId),
        Transfer(TokenId, AccountId, AccountId, TokenBalance),
        Approval(TokenId, AccountId, AccountId, TokenBalance),
        Mint(TokenId, AccountId, TokenBalance),
        Burn(TokenId, AccountId, TokenBalance),
    }
);

pub trait Trait: balances::Trait + system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as TokenStorage {
        Count get(count): TokenId;

        TokenDefaultSymbol get(token_default): Token = Token{id: 0, symbol: Vec::from("TOKEN")};
        TokenInfo get(token_info): map(TokenId) => Token;
        TokenIds get(token_id_by_symbol): map Vec<u8> => TokenId;
        TokenSymbol get(token_symbol_by_id): map TokenId => Vec<u8>;
        TotalSupply get(total_supply): map TokenId => TokenBalance;
        Balance get(balance_of): map (TokenId, T::AccountId) => TokenBalance;
        Allowance get(allowance_of): map (TokenId, T::AccountId, T::AccountId) => TokenBalance;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;

        // Burn tokens
        fn burn(origin, id: TokenId, from: T::AccountId, #[compact] amount: TokenBalance) -> Result {
            ensure_signed(origin)?;

            Self::_burn(id, from.clone(), amount)?;

            Ok(())
        }

        fn mint(origin, recepient: T::AccountId, #[compact] amount: TokenBalance, token: Vec<u8>) -> Result{
            let validator = ensure_signed(origin)?;
            Self::check_token_exist(validator, &token)?;

            let id = <TokenIds<T>>::get(&token);
            Self::_mint(id, recepient, amount)?;

            Ok(())
        }

        fn transfer(origin,
            #[compact] id: TokenId,
            to: <T::Lookup as StaticLookup>::Source,
            #[compact] amount: TokenBalance
        ) -> Result{
            let sender = ensure_signed(origin)?;
            let to = T::Lookup::lookup(to)?;
            ensure!(!amount.is_zero(), "transfer amount should be non-zero");

            Self::make_transfer(id, sender, to, amount)?;
            Ok(())
        }

        fn approve(origin,
            #[compact] id: TokenId,
            spender: <T::Lookup as StaticLookup>::Source,
            #[compact] value: TokenBalance
        ) -> Result{
            let sender = ensure_signed(origin)?;
            let spender = T::Lookup::lookup(spender)?;

            <Allowance<T>>::insert((id, sender.clone(), spender.clone()), value);

            Self::deposit_event(RawEvent::Approval(id, sender, spender, value));
            Ok(())
        }

        fn transfer_from(origin,
            #[compact] id: TokenId,
            from: T::AccountId,
            to: T::AccountId,
            #[compact] value: TokenBalance
        ) -> Result{
            let sender = ensure_signed(origin)?;
            let allowance = Self::allowance_of((id, from.clone(), sender.clone()));

            let updated_allowance = allowance.checked_sub(value).ok_or("underflow in calculating allowance")?;

            Self::make_transfer(id, from.clone(), to.clone(), value)?;

            <Allowance<T>>::insert((id, from, sender), updated_allowance);
            Ok(())
        }

    }
}

impl<T: Trait> Module<T> {
    pub fn _burn(id: TokenId, from: T::AccountId, amount: TokenBalance) -> Result {
        ensure!(
            Self::total_supply(id) > amount,
            "Cannot burn more than total supply"
        );

        let old_balance = <Balance<T>>::get((id, from.clone()));
        ensure!(
            old_balance > TokenBalance::zero(),
            "Cannot burn with zero balance"
        );

        let next_balance = old_balance
            .checked_sub(amount)
            .ok_or("underflow subtracting from balance burn")?;
        let next_total = Self::total_supply(id)
            .checked_sub(amount)
            .ok_or("underflow subtracting from total supply")?;
        <Balance<T>>::insert((id, from.clone()), next_balance);
        <TotalSupply<T>>::insert(id, next_total);

        Self::deposit_event(RawEvent::Burn(id, from, amount));

        Ok(())
    }
    pub fn _mint(id: TokenId, recepient: T::AccountId, amount: TokenBalance) -> Result {
        ensure!(!amount.is_zero(), "amount should be non-zero");

        let old_balance = <Balance<T>>::get((id, recepient.clone()));
        let next_balance = old_balance
            .checked_add(amount)
            .ok_or("overflow adding to balance")?;
        let next_total = Self::total_supply(id)
            .checked_add(amount)
            .ok_or("overflow adding to total supply")?;

        <Balance<T>>::insert((id, recepient.clone()), next_balance);
        <TotalSupply<T>>::insert(id, next_total);

        Self::deposit_event(RawEvent::Mint(id, recepient, amount));
        Ok(())
    }

    pub fn check_token_exist(validator: T::AccountId, token: &[u8]) -> Result {
        if !<TokenIds<T>>::exists(token.to_vec()) {
            Self::validate_name(token)?;
            Self::create_token(validator, &token.to_vec())?;
        }

        Ok(())
    }

    fn create_token(owner: T::AccountId, token: &Vec<u8>) -> Result {
        let next_id = match <Count<T>>::get() {
            0u32 => 0u32,
            count => count
                .checked_add(One::one())
                .ok_or("overflow when adding new token")?,
        };

        <Count<T>>::mutate(|n| *n = if next_id == 0u32 { 1 } else { next_id });
        <TokenIds<T>>::insert(token.clone(), &next_id);
        <TokenSymbol<T>>::insert(&next_id, token.clone());

        let next_token = Token {
            id: next_id,
            symbol: token.clone(),
        };

        <TokenInfo<T>>::insert(next_id, next_token);

        Self::deposit_event(RawEvent::NewToken(<Count<T>>::get(), owner.clone()));

        Ok(())
    }

    fn make_transfer(
        id: TokenId,
        from: T::AccountId,
        to: T::AccountId,
        amount: TokenBalance,
    ) -> Result {
        let from_balance = Self::balance_of((id, from.clone()));
        ensure!(from_balance >= amount, "user does not have enough tokens");

        <Balance<T>>::insert((id, from.clone()), from_balance - amount);
        <Balance<T>>::mutate((id, to.clone()), |balance| *balance += amount);

        Self::deposit_event(RawEvent::Transfer(id, from, to, amount));

        Ok(())
    }
    fn validate_name(name: &[u8]) -> Result {
        if name.len() > 10 {
            return Err("the token symbol is too long");
        }
        if name.len() < 3 {
            return Err("the token symbol is too short");
        }

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

    const TOKEN_NAME: &[u8; 5] = b"TOKEN";
    const TOKEN_SHORT_NAME: &[u8; 1] = b"T";
    const TOKEN_LONG_NAME: &[u8; 34] = b"nobody_really_want_such_long_token";
    const USER1: u64 = 1;
    const USER2: u64 = 2;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        let mut r = system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap()
            .0;

        r.extend(
            balances::GenesisConfig::<Test> {
                balances: vec![(USER1, 100000), (USER2, 300000)],
                vesting: vec![],
                transaction_base_fee: 0,
                transaction_byte_fee: 0,
                existential_deposit: 500,
                transfer_fee: 0,
                creation_fee: 0,
            }
            .build_storage()
            .unwrap()
            .0,
        );

        r.into()
    }

    #[test]
    fn mint_new_token_works() {
        with_externalities(&mut new_test_ext(), || {
            assert_ok!(TokenModule::mint(
                Origin::signed(USER1),
                USER2,
                1000,
                TOKEN_NAME.to_vec()
            ));

            assert_eq!(TokenModule::balance_of((0, USER2)), 1000);
            assert_eq!(TokenModule::total_supply(0), 1000);
        })
    }

    #[test]
    fn mint_new_token_too_long() {
        with_externalities(&mut new_test_ext(), || {
            assert_eq!(TokenModule::count(), 0);
            assert_noop!(
                TokenModule::mint(Origin::signed(USER1), USER2, 1000, TOKEN_LONG_NAME.to_vec()),
                "the token symbol is too long"
            );
        })
    }

    #[test]
    fn mint_new_token_too_short() {
        with_externalities(&mut new_test_ext(), || {
            assert_eq!(TokenModule::count(), 0);
            assert_noop!(
                TokenModule::mint(
                    Origin::signed(USER1),
                    USER2,
                    1000,
                    TOKEN_SHORT_NAME.to_vec()
                ),
                "the token symbol is too short"
            );
        })
    }

    #[test]
    fn token_transfer_works() {
        with_externalities(&mut new_test_ext(), || {
            assert_ok!(TokenModule::mint(
                Origin::signed(USER1),
                USER2,
                1000,
                TOKEN_NAME.to_vec()
            ));

            assert_eq!(TokenModule::balance_of((0, USER2)), 1000);
            assert_ok!(TokenModule::transfer(Origin::signed(USER2), 0, USER1, 300));
            assert_eq!(TokenModule::balance_of((0, USER2)), 700);
            assert_eq!(TokenModule::balance_of((0, USER1)), 300);
        })
    }

    #[test]
    fn token_transfer_not_enough() {
        with_externalities(&mut new_test_ext(), || {
            assert_ok!(TokenModule::mint(
                Origin::signed(USER1),
                USER2,
                1000,
                TOKEN_NAME.to_vec()
            ));

            assert_eq!(TokenModule::balance_of((0, USER2)), 1000);
            assert_ok!(TokenModule::transfer(Origin::signed(USER2), 0, USER1, 300));
            assert_eq!(TokenModule::balance_of((0, USER2)), 700);
            assert_eq!(TokenModule::balance_of((0, USER1)), 300);

            assert_noop!(
                TokenModule::transfer(Origin::signed(USER2), 0, USER1, 1300),
                "user does not have enough tokens"
            );
        })
    }
}
