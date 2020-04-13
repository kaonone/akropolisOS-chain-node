use crate::types::{DaoId, Days, Rate, TokenId};
use frame_support::{
    decl_event, decl_module, decl_storage, dispatch::DispatchResult, weights::SimpleDispatchInfo,
    StorageValue,
};
use sp_std::prelude::Vec;
use system::ensure_signed;

pub trait Trait: balances::Trait + system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as marketplace {
        Something get(something): Option<u64>;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        #[weight = SimpleDispatchInfo::FixedNormal(10_000)]
        fn make_investment(origin, proposal_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            <Something>::put(proposal_id);

            Self::deposit_event(RawEvent::NewInvsetment(proposal_id, who));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn propose_investment(
        dao_id: DaoId,
        description: Vec<u8>,
        days: Days,
        rate: Rate,
        token: TokenId,
        price: T::Balance,
        value: T::Balance,
    ) -> DispatchResult {
        // TODO: do usefull stuff :D
        Self::deposit_event(RawEvent::ProposeInvestment(
            dao_id,
            description,
            days,
            rate,
            token,
            price,
            value,
        ));
        Ok(())
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Balance = <T as balances::Trait>::Balance,
    {
        NewInvsetment(u64, AccountId),
        ProposeInvestment(DaoId, Vec<u8>, Days, Rate, TokenId, Balance, Balance),
    }
);

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use frame_support::{
        assert_ok, impl_outer_origin, parameter_types, traits::Get, weights::Weight,
    };
    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        Perbill,
    };
    use std::cell::RefCell;

    pub type Balance = u128;

    thread_local! {
        static EXISTENTIAL_DEPOSIT: RefCell<u128> = RefCell::new(500);
    }

    impl_outer_origin! {
        pub enum Origin for Test {}
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
        type AccountData = balances::AccountData<u128>;
        type OnNewAccount = ();
        type OnKilledAccount = ();
    }

    impl balances::Trait for Test {
        type Balance = Balance;
        type DustRemoval = ();
        type Event = ();
        type ExistentialDeposit = ExistentialDeposit;
        type AccountStore = system::Module<Test>;
    }

    impl Trait for Test {
        type Event = ();
    }
    type Marketplace = Module<Test>;

    const DAO_DESC: &[u8; 10] = b"Desc-1234_";

    pub struct ExtBuilder {
        existential_deposit: u128,
    }

    impl Default for ExtBuilder {
        fn default() -> Self {
            Self {
                existential_deposit: 500,
            }
        }
    }

    impl ExtBuilder {
        pub fn set_associated_consts(&self) {
            EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = self.existential_deposit);
        }
        pub fn build(self) -> sp_io::TestExternalities {
            self.set_associated_consts();
            let mut storage = system::GenesisConfig::default()
                .build_storage::<Test>()
                .unwrap();

            let _ = balances::GenesisConfig::<Test> {
                balances: vec![
                    (2, 20000),
                    (3, 30000),
                    (4, 400000),
                    (11, 500),
                    (21, 2000),
                    (31, 2000),
                    (41, 2000),
                    (100, 2000),
                    (101, 2000),
                    // This allow us to have a total_payout different from 0.
                    (999, 1_000_000_000_000),
                ],
            }
            .assimilate_storage(&mut storage);

            let ext = sp_io::TestExternalities::from(storage);
            ext
        }
    }

    #[test]
    fn make_investment_should_work() {
        ExtBuilder::default().build().execute_with(|| {
            assert_ok!(Marketplace::make_investment(Origin::signed(31), 42));
            assert_eq!(Marketplace::something(), Some(42));
        });
    }

    #[test]
    fn propose_tokenized_investment_should_work() {
        const DAO_ID: DaoId = 11;
        const DAYS: Days = 181;
        const RATE: Rate = 1000;
        const TOKEN: TokenId = 0;
        const TOKEN_PRICE: Balance = 1;
        const VALUE: u128 = 42;

        ExtBuilder::default().build().execute_with(|| {
            assert_ok!(Marketplace::propose_investment(
                DAO_ID,
                DAO_DESC.to_vec(),
                DAYS,
                RATE,
                TOKEN,
                TOKEN_PRICE,
                VALUE
            ));
        });
    }
}
