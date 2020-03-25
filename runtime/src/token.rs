/// runtime module implementing the ERC20 token factory API
/// You can use mint to create tokens or burn created tokens
/// and transfer tokens on substrate side freely or operate with total_supply
///
use crate::types::{Token, TokenBalance, TokenId};
use sp_std::prelude::Vec;
use sp_runtime::traits::{StaticLookup, Zero};
use frame_support::{
    decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure, StorageMap,
};
use system::{self, ensure_signed};

type Result<T> = core::result::Result<T, &'static str>;

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        Transfer(AccountId, AccountId, TokenBalance),
        Approval(AccountId, AccountId, TokenBalance),
        Mint(AccountId, TokenBalance),
        Burn(AccountId, TokenBalance),
    }
);

pub trait Trait: balances::Trait + system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as TokenStorage {
        pub Count get(fn count): TokenId;
        pub Locked get(fn locked): map hasher(blake2_256) (TokenId, T::AccountId) => TokenBalance;

        pub Tokens get(fn tokens) build(|config: &GenesisConfig| {
            config.tokens.clone().into_iter().enumerate()
            .map(|(i, t): (usize, Token)| (i as u32, t)).collect::<Vec<_>>()
        }): map hasher(blake2_256) TokenId => Token;
        pub TokenIds get(fn token_id_by_symbol) build(|config: &GenesisConfig| {
            config.tokens.clone().into_iter().map(|t: Token| (t.symbol, t.id)).collect::<Vec<_>>()
        }): map hasher(blake2_256) Vec<u8> => TokenId;
        pub TokenSymbol get(fn token_symbol_by_id) build(|config: &GenesisConfig| {
            config.tokens.clone().into_iter().enumerate()
            .map(|(i, t): (usize, Token)| (i as u32, t.symbol)).collect::<Vec<_>>()
        }): map hasher(blake2_256) TokenId => Vec<u8>;
        pub TotalSupply get(fn total_supply): map hasher(blake2_256) TokenId => TokenBalance;
        pub Balance get(fn balance_of): map hasher(blake2_256) (TokenId, T::AccountId) => TokenBalance;
        pub Allowance get(fn allowance_of): map hasher(blake2_256) (TokenId, T::AccountId, T::AccountId) => TokenBalance;
    }
    add_extra_genesis{
        config(tokens): Vec<Token>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        // ( ! ): can be called directly
        // ( ? ): do we even need this?
        fn burn(origin, from: T::AccountId, #[compact] amount: TokenBalance) -> DispatchResult {
            ensure_signed(origin)?;
            // TODO: replace this by adding it to extrinsics call    ^
            let token = <Tokens<T>>::get(0);
            Self::check_token_exist(&token.symbol)?;
            Self::_burn(0, from.clone(), amount)?;
            Self::deposit_event(RawEvent::Burn(from, amount));
            Ok(())
        }

        // ( ! ): can be called directly
        // ( ? ): do we even need this?
        fn mint(origin, to: T::AccountId, #[compact] amount: TokenBalance) -> DispatchResult{
            ensure_signed(origin)?;
            // TODO: replace this by adding it to extrinsics call    ^
            let token = <Tokens<T>>::get(0);
            Self::check_token_exist(&token.symbol)?;
            Self::_mint(token.id, to.clone(), amount)?;
            Self::deposit_event(RawEvent::Mint(to.clone(), amount));
            Ok(())
        }

        // TODO: decide whether we need it from outside
        // fn create_token(origin, token: Vec<u8>) -> DispatchResult {
        //     ensure_signed(origin)?;
        //     Self::check_token_exist(&token)
        // }

        fn transfer(origin,
            to: <T::Lookup as StaticLookup>::Source,
            #[compact] amount: TokenBalance
        ) -> DispatchResult{
            let sender = ensure_signed(origin)?;
            let to = T::Lookup::lookup(to)?;
            ensure!(!amount.is_zero(), "Transfer Amount should be non-zero");

            let id = <Tokens<T>>::get(0).id;
            Self::make_transfer(id, sender, to, amount)?;
            Ok(())
        }

        fn approve(origin,
            spender: <T::Lookup as StaticLookup>::Source,
            #[compact] value: TokenBalance
        ) -> DispatchResult{
            let sender = ensure_signed(origin)?;
            let spender = T::Lookup::lookup(spender)?;

            let id = <Tokens<T>>::get(0).id;
            <Allowance<T>>::insert((id,sender.clone(), spender.clone()), value);

            Self::deposit_event(RawEvent::Approval(sender, spender, value));
            Ok(())
        }

        fn transfer_from(origin,
            from: T::AccountId,
            to: T::AccountId,
            #[compact] value: TokenBalance
        ) -> DispatchResult{
            let sender = ensure_signed(origin)?;
            let id = <Tokens<T>>::get(0).id;
            let allowance = Self::allowance_of((id, from.clone(), sender.clone()));

            let updated_allowance = allowance.checked_sub(value).ok_or("Underflow in calculating allowance")?;


            let id = <Tokens<T>>::get(0).id;
            Self::make_transfer(id, from.clone(), to.clone(), value)?;

            <Allowance<T>>::insert((id, from, sender), updated_allowance);
            Ok(())
        }

    }
}

impl<T: Trait> Module<T> {
    pub fn _burn(token_id: TokenId, from: T::AccountId, amount: TokenBalance) -> Result<()> {
        ensure!(
            Self::total_supply(0) >= amount,
            "Cannot burn more than total supply"
        );

        let free_balance = <Balance<T>>::get((token_id, from.clone()))
            - <Locked<T>>::get((token_id, from.clone()));
        ensure!(
            free_balance > TokenBalance::zero(),
            "Cannot burn with zero balance"
        );
        ensure!(free_balance >= amount, "Not enough because of locked funds");

        let next_balance = free_balance
            .checked_sub(amount)
            .ok_or("Underflow subtracting from balance burn")?;
        let next_total = Self::total_supply(0)
            .checked_sub(amount)
            .ok_or("Underflow subtracting from total supply")?;

        <Balance<T>>::insert((token_id, from.clone()), next_balance);
        <TotalSupply>::insert(token_id, next_total);

        Ok(())
    }
    pub fn _mint(token_id: TokenId, to: T::AccountId, amount: TokenBalance) -> Result<()> {
        ensure!(!amount.is_zero(), "Amount should be non-zero");

        let old_balance = <Balance<T>>::get((token_id, to.clone()));
        let next_balance = old_balance
            .checked_add(amount)
            .ok_or("Overflow adding to balance")?;
        let next_total = Self::total_supply(0)
            .checked_add(amount)
            .ok_or("Overflow adding to total supply")?;

        <Balance<T>>::insert((token_id, to.clone()), next_balance);
        <TotalSupply>::insert(token_id, next_total);

        Ok(())
    }

    fn make_transfer(
        token_id: TokenId,
        from: T::AccountId,
        to: T::AccountId,
        amount: TokenBalance,
    ) -> Result<()> {
        let from_balance = <Balance<T>>::get((token_id, from.clone()));
        ensure!(from_balance >= amount, "User does not have enough tokens");
        let free_balance = <Balance<T>>::get((token_id, from.clone()))
            - <Locked<T>>::get((token_id, from.clone()));
        ensure!(free_balance >= amount, "Not enough because of locked funds");

        <Balance<T>>::insert((token_id, from.clone()), from_balance - amount);
        <Balance<T>>::mutate((token_id, to.clone()), |balance| *balance += amount);

        Self::deposit_event(RawEvent::Transfer(from, to, amount));

        Ok(())
    }
    pub fn lock(token_id: TokenId, account: T::AccountId, amount: TokenBalance) -> Result<()> {
        //TODO: substract this amount from the main balance?
        //              Balance: 1000, Locked: 0
        // lock(400) => Balance: 1000, Locked: 400 or
        // lock(400) => Balance: 600, Locked: 400
        <Locked<T>>::insert((token_id, account.clone()), amount);

        Ok(())
    }
    pub fn unlock(token_id: TokenId, account: &T::AccountId, amount: TokenBalance) -> Result<()> {
        //TODO: add this amount to the main balance?
        //                Balance: 1000, Locked: 400
        // unlock(400) => Balance: 1000, Locked: 0 or
        // unlock(400) => Balance: 1400, Locked: 0
        let balance = <Locked<T>>::get((token_id, account.clone()));
        let new_balance = balance
            .checked_sub(amount)
            .expect("Underflow while unlocking. Check if user has enough locked funds.");
        match balance - amount {
            0 => <Locked<T>>::remove((token_id, account.clone())),
            _ => <Locked<T>>::insert((token_id, account.clone()), new_balance),
        }
        Ok(())
    }
    // Token management
    // Add new or do nothing
    pub fn check_token_exist(token: &Vec<u8>) -> Result<()> {
        if !<TokenIds<T>>::contains_key(token.clone()) {
            Self::validate_name(token)
        } else {
            Ok(())
        }
    }

    fn validate_name(name: &[u8]) -> Result<()> {
        if name.len() > 10 {
            return Err("The token symbol is too long");
        }
        if name.len() < 3 {
            return Err("The token symbol is too short");
        }

        Ok(())
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use sp_core::{Blake2Hasher, H256};
    use runtime_io::with_externalities;
    use sp_runtime::{
        testing::{Digest, DigestItem, Header},
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    };
    use frame_support::{assert_noop, assert_ok, impl_outer_origin};

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

    // const TOKEN_NAME: &[u8; 4] = b"DOOM";
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
                transaction_base_fee: 1,
                transaction_byte_fee: 1,
                existential_deposit: 500,
                transfer_fee: 1,
                creation_fee: 1,
            }
            .build_storage()
            .unwrap()
            .0,
        );

        r.extend(
            GenesisConfig::<Test> {
                tokens: vec![Token {
                    id: 0,
                    decimals: 18,
                    symbol: Vec::from("TOKEN"),
                }],
                _genesis_phantom_data: Default::default(),
            }
            .build_storage()
            .unwrap()
            .0,
        );

        r.into()
    }

    #[test]
    fn new_token_mint_works() {
        with_externalities(&mut new_test_ext(), || {
            assert_ok!(TokenModule::check_token_exist(
                &TokenModule::tokens(0).symbol
            ));
            assert_ok!(TokenModule::_mint(0, USER2, 1000));
            assert_eq!(TokenModule::balance_of((0, USER2)), 1000);
            assert_eq!(TokenModule::total_supply(0), 1000);
        })
    }

    #[test]
    fn new_token_mint_and_burn_works() {
        with_externalities(&mut new_test_ext(), || {
            assert_ok!(TokenModule::check_token_exist(
                &TokenModule::tokens(0).symbol
            ));
            assert_ok!(TokenModule::_mint(0, USER2, 1000));
            assert_eq!(TokenModule::balance_of((0, USER2)), 1000);

            assert_ok!(TokenModule::_burn(0, USER2, 1000));
            assert_eq!(TokenModule::balance_of((0, USER2)), 0);
        })
    }

    #[test]
    fn token_transfer_works() {
        with_externalities(&mut new_test_ext(), || {
            assert_ok!(TokenModule::_mint(0, USER2, 1000));

            assert_eq!(TokenModule::balance_of((0, USER2)), 1000);
            assert_ok!(TokenModule::transfer(Origin::signed(USER2), USER1, 300));
            assert_eq!(TokenModule::balance_of((0, USER2)), 700);
            assert_eq!(TokenModule::balance_of((0, USER1)), 300);
        })
    }
    #[test]
    fn token_lock_works() {
        with_externalities(&mut new_test_ext(), || {
            assert_ok!(TokenModule::_mint(0, USER2, 1000));

            assert_eq!(TokenModule::balance_of((0, USER2)), 1000);
            assert_ok!(TokenModule::lock(0, USER2, 400));
            assert_eq!(TokenModule::locked((0, USER2)), 400);
        })
    }

    #[test]
    fn token_unlock_works() {
        with_externalities(&mut new_test_ext(), || {
            assert_ok!(TokenModule::_mint(0, USER2, 1000));

            assert_eq!(TokenModule::balance_of((0, USER2)), 1000);
            assert_ok!(TokenModule::lock(0, USER2, 400));
            assert_eq!(TokenModule::locked((0, USER2)), 400);
            assert_ok!(TokenModule::unlock(0, &USER2, 400));
            assert_eq!(TokenModule::locked((0, USER2)), 0);
        })
    }

    #[test]
    fn token_transfer_not_enough() {
        with_externalities(&mut new_test_ext(), || {
            assert_ok!(TokenModule::_mint(0, USER2, 1000));

            assert_eq!(TokenModule::balance_of((0, USER2)), 1000);
            assert_ok!(TokenModule::transfer(Origin::signed(USER2), USER1, 300));
            assert_eq!(TokenModule::balance_of((0, USER2)), 700);
            assert_eq!(TokenModule::balance_of((0, USER1)), 300);
            assert_eq!(TokenModule::locked((0, USER2)), 0);
            assert_noop!(
                TokenModule::transfer(Origin::signed(USER2), USER1, 1300),
                "User does not have enough tokens"
            );
        })
    }
    #[test]
    fn token_transfer_burn_works() {
        with_externalities(&mut new_test_ext(), || {
            assert_ok!(TokenModule::_mint(0, USER2, 1000));
            assert_eq!(TokenModule::balance_of((0, USER2)), 1000);

            assert_ok!(TokenModule::_burn(0, USER2, 300));
            assert_eq!(TokenModule::balance_of((0, USER2)), 700);
        })
    }
    #[test]
    fn token_transfer_burn_all_works() {
        with_externalities(&mut new_test_ext(), || {
            assert_ok!(TokenModule::_mint(0, USER2, 1000));
            assert_eq!(TokenModule::balance_of((0, USER2)), 1000);

            assert_ok!(TokenModule::_burn(0, USER2, 1000));
            assert_eq!(TokenModule::balance_of((0, USER2)), 0);
        })
    }

    #[test]
    fn new_token_symbol_len_failed() {
        with_externalities(&mut new_test_ext(), || {
            assert_noop!(
                TokenModule::validate_name(&TOKEN_SHORT_NAME.to_vec()),
                "The token symbol is too short"
            );
            assert_noop!(
                TokenModule::validate_name(&TOKEN_LONG_NAME.to_vec()),
                "The token symbol is too long"
            );
            assert_eq!(TokenModule::count(), 0);
        })
    }
}
