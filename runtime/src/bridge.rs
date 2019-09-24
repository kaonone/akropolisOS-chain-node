/// runtime module implementing Substrate side of AkropolisOS token exchange bridge
/// You can use mint to create tokens backed by locked funds on Ethereum side
/// and transfer tokens on substrate side freely
///
use crate::token;
use crate::types::{MemberId, ProposalId, TokenBalance};
use parity_codec::{Decode, Encode};
use runtime_primitives::traits::{Hash};
use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, ensure, StorageMap, StorageValue,
};
use system::{self, ensure_signed};


#[derive(Encode, Decode, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BridgeProposal<AccountId, Hash> {
    proposal_id: ProposalId,
    action: Action<AccountId, Hash>,
    open: bool,
    votes: MemberId,
}

impl<A, H> Default for BridgeProposal<A, H>
where
    A: Default,
{
    fn default() -> Self {
        BridgeProposal {
            proposal_id: ProposalId::default(),
            action: Action::EmptyAction,
            open: true,
            votes: MemberId::default(),
        }
    }
}

#[derive(Encode, Decode, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum Action<AccountId, Hash> {
    EmptyAction,
    Ethereum2Substrate(Hash, AccountId, TokenBalance),
    Substrate2Ethereum(AccountId, Hash, TokenBalance),
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Hash = <T as system::Trait>::Hash,
    {
        IntentToMint(AccountId, TokenBalance),
        IntentToBurn(Hash),
        ProposalIsAccepted(ProposalId),
        ProposalIsRejected(ProposalId),
    }
);

pub trait Trait: token::Trait + system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as BridgeStorage {
        BridgeProposals get(proposals): map ProposalId => BridgeProposal<T::AccountId, T::Hash>;
        BridgeProposalsCount get(bridge_proposals_count): ProposalId;

        OpenBridgeProposalsHashes get(open_proposal_index_by_hash): map(T::Hash) => ProposalId;
        OpenBridgeProposalsHashesIndex get(open_proposal_hash_by_index): map(ProposalId) => T::Hash;

        EthereumAdressHashes get(ethereum_address): map(ProposalId) => T::Hash;
        ValidatorsCount get(validators_count) config(): usize = 3;
        ValidatorsAccounts get(validators_accounts): map MemberId => T::AccountId;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;

        fn set_transfer(origin,
            from: T::AccountId,
            to: T::Hash, //Ethereum address
            #[compact] amount: TokenBalance
        )-> Result{
            ensure_signed(origin)?;

            let proposal_hash = ("set_transfer", &from, amount).using_encoded(<T as system::Trait>::Hashing::hash);
            let action = Action::Substrate2Ethereum(from.clone(), to, amount);
            let event = RawEvent::IntentToBurn(proposal_hash);
            Self::get_proposal_id_checked(proposal_hash, event, action)?;

            let proposal_id = <OpenBridgeProposalsHashes<T>>::get(proposal_hash);
            Self::_vote(proposal_id, true)?;
            <EthereumAdressHashes<T>>::insert(proposal_id, to);
            Ok(())
        }

        fn eth2substrate(origin,
            message_id: T::Hash,
            from: T::Hash, //Ethereum address
            to: T::AccountId,
            // to_hash: T::Hash,
            #[compact] amount: TokenBalance
        )-> Result {
            ensure_signed(origin)?;
            // let to = T::AccountId::from_512(to_hash);

            let proposal_hash = message_id.using_encoded(<T as system::Trait>::Hashing::hash);
            let action = Action::Ethereum2Substrate(from, to.clone(), amount);
            let event = RawEvent::IntentToMint(to, amount);
            Self::get_proposal_id_checked(proposal_hash, event, action)?;

            let proposal_id = <OpenBridgeProposalsHashes<T>>::get(proposal_hash);
            Self::_vote(proposal_id, true)?;
            <EthereumAdressHashes<T>>::insert(proposal_id, from);
            Ok(())
        }
        
        fn sign(origin, message_id: T::Hash) -> Result {
            ensure_signed(origin)?;
            let id = <OpenBridgeProposalsHashes<T>>::get(message_id);

            Self::_vote(id, true)?;

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn _vote(proposal_id: ProposalId, vote: bool) -> Result {
        let mut proposal = <BridgeProposals<T>>::get(proposal_id);
        ensure!(proposal.open, "This proposal is not open");

        if vote {
            proposal.votes += 1;
        }

        let proposal_is_accepted = Self::votes_are_enough(proposal.votes);
        let all_validators_voted = proposal.votes == Self::validators_count() as u64;

        if proposal_is_accepted {
            Self::execute_proposal(proposal.clone())?;
        }

        if proposal_is_accepted || all_validators_voted {
            Self::close_proposal(proposal.clone());
        } else {
            <BridgeProposals<T>>::insert(proposal_id, proposal);
        }

        match (proposal_is_accepted, all_validators_voted) {
            (true, _) => Self::deposit_event(RawEvent::ProposalIsAccepted(proposal_id)),
            (_, true) => Self::deposit_event(RawEvent::ProposalIsRejected(proposal_id)),
            (_, _) => (),
        }

        Ok(())
    }
    fn get_proposal_id_checked(
        proposal_hash: T::Hash,
        event: RawEvent<T::AccountId, T::Hash>,
        action: Action<T::AccountId, T::Hash>,
    ) -> Result {
        match <OpenBridgeProposalsHashes<T>>::exists(proposal_hash) {
            true => Ok(()),
            false => {
                Self::create_proposal(proposal_hash, action)?;
                Self::deposit_event(event);
                Ok(())
            }
        }
    }
    fn close_proposal(mut proposal: BridgeProposal<T::AccountId, T::Hash>) {
        let proposal_id = proposal.proposal_id.clone();
        proposal.open = false;
        let proposal_hash = <OpenBridgeProposalsHashesIndex<T>>::get(proposal_id);

        <BridgeProposals<T>>::insert(proposal_id, proposal);
        <OpenBridgeProposalsHashes<T>>::remove(proposal_hash);
        <OpenBridgeProposalsHashesIndex<T>>::remove(proposal_id);
    }

    fn votes_are_enough(votes: MemberId) -> bool {
        votes as f64 / Self::validators_count() as f64 >= 0.51
    }

    fn execute_proposal(proposal: BridgeProposal<T::AccountId, T::Hash>) -> Result {
        match proposal.action {
            Action::Substrate2Ethereum(from, to, amount) => {
                <token::Module<T>>::_burn(from, to, amount)
            }
            Action::Ethereum2Substrate(from, to, amount) => {
                <token::Module<T>>::_mint(from, to, amount)
            }
            Action::EmptyAction => Ok(()),
        }
    }
    fn create_proposal(proposal_hash: T::Hash, action: Action<T::AccountId, T::Hash>) -> Result {
        ensure!(
            !<OpenBridgeProposalsHashes<T>>::exists(proposal_hash),
            "This proposal already open"
        );
        let proposal_id = <BridgeProposalsCount<T>>::get();
        let bridge_proposals_count = <BridgeProposalsCount<T>>::get();
        let new_bridge_proposals_count = bridge_proposals_count
            .checked_add(1)
            .ok_or("Overflow adding a new bridge proposal")?;

        let proposal = BridgeProposal {
            proposal_id,
            action,
            open: true,
            votes: MemberId::default(),
        };

        <BridgeProposals<T>>::insert(proposal_id, proposal);
        <BridgeProposalsCount<T>>::mutate(|count| *count += new_bridge_proposals_count);
        <OpenBridgeProposalsHashes<T>>::insert(proposal_hash, proposal_id);
        <OpenBridgeProposalsHashesIndex<T>>::insert(proposal_id, proposal_hash);

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
    use support::{assert_ok, impl_outer_origin};

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
    impl token::Trait for Test {
        type Event = ();
    }
    impl Trait for Test {
        type Event = ();
    }

    type BridgeModule = Module<Test>;
    type TokenModule = token::Module<Test>;

    const MESSAGE_ID: &[u8; 32] = b"0x5617efe391571b5dc8230db92ba65b";
    const ETH_ADDRESS: &[u8; 32] = b"0x00b46c2526ebb8f4c9e4674d262e75";
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
    fn token_eth2sub_mint_works() {
        with_externalities(&mut new_test_ext(), || {
            let message_id = H256::from(MESSAGE_ID);
            let eth_address = H256::from(ETH_ADDRESS);

            assert_ok!(BridgeModule::eth2substrate(
                Origin::signed(USER2),
                message_id,
                eth_address,
                USER2,
                1000
            ));
            assert_ok!(BridgeModule::eth2substrate(
                Origin::signed(USER1),
                message_id,
                eth_address,
                USER2,
                1000
            ));
            assert_eq!(TokenModule::balance_of(USER2), 1000);
            assert_eq!(TokenModule::total_supply(), 1000);
        })
    }

    #[test]
    fn token_sub2eth_burn_works() {
        with_externalities(&mut new_test_ext(), || {
            let message_id = H256::from(MESSAGE_ID);
            let eth_address = H256::from(ETH_ADDRESS);

            assert_ok!(BridgeModule::eth2substrate(
                Origin::signed(USER2),
                message_id,
                eth_address,
                USER2,
                1000
            ));
            assert_ok!(BridgeModule::eth2substrate(
                Origin::signed(USER1),
                message_id,
                eth_address,
                USER2,
                1000
            ));
            assert_eq!(TokenModule::balance_of(USER2), 1000);
            assert_eq!(TokenModule::total_supply(), 1000);

            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER1),
                USER2,
                eth_address,
                500
            ));
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER2),
                USER2,
                eth_address,
                500
            ));
            assert_eq!(TokenModule::balance_of(USER2), 500);
            assert_eq!(TokenModule::total_supply(), 500);
        })
    }
}
