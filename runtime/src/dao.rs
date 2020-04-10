/// Pallet implementing DAO module.
///
/// Create DAO providing address of a future organization.
/// Make loans in native currency with voting.
/// Make loans in other tokens with fetched prices from oracle.
/// Add\remove members with voting.
///
use codec::Encode;
use frame_support::{
    decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::{
        Currency, ExistenceRequirement, Get, LockIdentifier, LockableCurrency, WithdrawReasons,
    },
    StorageMap, StorageValue,
};
use num_traits::ops::checked::CheckedSub;
use sp_runtime::traits::{Hash, Zero};
use sp_std::prelude::Vec;
use system::ensure_signed;

use crate::types::*;
use crate::{marketplace, price_oracle, token};

const LOCK_NAME: LockIdentifier = *b"dao_lock";
const MINIMUM_VOTE_TIOMEOUT: u32 = 30; // ~5 min
const MAXIMUM_VOTE_TIMEOUT: u32 = 3 * 30 * 24 * 60 * 6; // ~90 days

pub trait Trait:
    marketplace::Trait
    + token::Trait
    + balances::Trait
    + timestamp::Trait
    + system::Trait
    + price_oracle::Trait
{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as Dao {
        Daos get(fn daos): map hasher(opaque_blake2_256) DaoId => Dao<T::AccountId>;
        DaosCount get(fn daos_count): Count;
        DaoNames get(fn dao_names): map hasher(opaque_blake2_256) T::Hash => DaoId;
        DaoAddresses get(fn dao_addresses): map hasher(opaque_blake2_256) T::AccountId => DaoId;
        DaoTimeouts get(fn dao_timeouts): map hasher(opaque_blake2_256) DaoId => T::BlockNumber;
        DaoMaximumNumberOfMembers get(fn dao_maximum_number_of_members): map hasher(opaque_blake2_256) DaoId => MemberId;
        Address get(fn address): map hasher(opaque_blake2_256) DaoId => T::AccountId;

        MinumumNumberOfMebers get(fn minimum_number_of_members) config(): MemberId = 1;
        MaximumNumberOfMebers get(fn maximum_number_of_members) config(): MemberId = 4;
        Members get(fn members): map hasher(opaque_blake2_256) (DaoId, MemberId) => T::AccountId;
        MembersCount get(fn members_count): map hasher(opaque_blake2_256) DaoId => MemberId;
        DaoMembers get(fn dao_members): map hasher(opaque_blake2_256) (DaoId, T::AccountId) => MemberId;

        DaoProposals get(fn dao_proposals): map hasher(opaque_blake2_256) (DaoId, ProposalId) => Proposal<DaoId, T::AccountId, T::Balance, T::BlockNumber, VotesCount>;
        DaoProposalsCount get(fn dao_proposals_count): map hasher(opaque_blake2_256) DaoId => ProposalId;
        DaoProposalsIndex get(fn dao_proposals_index): map hasher(opaque_blake2_256) ProposalId => DaoId;

        DaoProposalsVotes get(fn dao_proposals_votes): map hasher(opaque_blake2_256) (DaoId, ProposalId, MemberId) => T::AccountId;
        DaoProposalsVotesCount get(fn dao_proposals_votes_count): map hasher(opaque_blake2_256) (DaoId, ProposalId) => MemberId;
        DaoProposalsVotesIndex get(fn dao_proposals_votes_index): map hasher(opaque_blake2_256) (DaoId, ProposalId, T::AccountId) => MemberId;

        OpenDaoProposalsLimit get(fn open_proposals_per_block) config(): u32 = 2;
        OpenDaoProposals get(fn open_dao_proposals): map hasher(opaque_blake2_256) T::BlockNumber => Vec<ProposalId>;
        OpenDaoProposalsIndex get(fn open_dao_proposals_index): map hasher(opaque_blake2_256) ProposalId => T::BlockNumber;
        OpenDaoProposalsHashes get(fn open_dao_proposals_hashes): map hasher(opaque_blake2_256) T::Hash => ProposalId;
        OpenDaoProposalsHashesIndex get(fn open_dao_proposals_hashes_index): map hasher(opaque_blake2_256) ProposalId => T::Hash;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        pub fn create(origin, address: T::AccountId, name: Vec<u8>, description: Vec<u8>) -> DispatchResult {
            let founder = ensure_signed(origin)?;

            let daos_count = <DaosCount>::get();
            let new_daos_count = daos_count
                .checked_add(1)
                .ok_or("Overflow adding a new dao")?;
            let name_hash = (&name).using_encoded(<T as system::Trait>::Hashing::hash);
            let zero = <T::Balance>::zero();

            ensure!(founder != address, "Founder address matches DAO address");
            Self::validate_name(&name)?;
            Self::validate_description(&description)?;
            ensure!(!<DaoAddresses<T>>::contains_key(&address), "This DAO address already busy");
            ensure!(!<DaoNames<T>>::contains_key(&name_hash), "This DAO name already exists");
            ensure!(<balances::Module<T>>::reserved_balance(&address) == zero, "Reserved balance of DAO address is not 0");

            let new_dao = Dao {
                address: address.clone(),
                name: name.clone(),
                description,
                founder: founder.clone()
            };
            let dao_id = daos_count;

            let dao_deposit = <T as balances::Trait>::ExistentialDeposit::get();

            <balances::Module<T> as Currency<_>>::transfer(&founder, &address, dao_deposit, ExistenceRequirement::KeepAlive)?;
            Self::set_account_lock(&address);

            <Daos<T>>::insert(dao_id, new_dao);
            <DaosCount>::put(new_daos_count);
            <DaoNames<T>>::insert(name_hash, dao_id);
            <DaoAddresses<T>>::insert(&address, dao_id);
            <DaoTimeouts<T>>::insert(dao_id, T::BlockNumber::from(MINIMUM_VOTE_TIOMEOUT));
            <DaoMaximumNumberOfMembers>::insert(dao_id, Self::maximum_number_of_members());
            <Address<T>>::insert(dao_id, &address);
            <Members<T>>::insert((dao_id, 0), &founder);
            <MembersCount>::insert(dao_id, 1);
            <DaoMembers<T>>::insert((dao_id, founder.clone()), 0);

            Self::deposit_event(RawEvent::DaoCreated(address, founder, name));
            Ok(())
        }

        pub fn propose_to_add_member(origin, dao_id: DaoId) -> DispatchResult {
            let candidate = ensure_signed(origin)?;

            let proposal_hash = ("propose_to_add_member", &candidate, dao_id)
                .using_encoded(<T as system::Trait>::Hashing::hash);
            let voting_deadline = <system::Module<T>>::block_number() + <DaoTimeouts<T>>::get(dao_id);
            let mut open_proposals = Self::open_dao_proposals(voting_deadline);

            ensure!(<Daos<T>>::contains_key(dao_id), "This DAO not exists");
            ensure!(!<DaoMembers<T>>::contains_key((dao_id, candidate.clone())), "You already are a member of this DAO");
            ensure!(!<DaoAddresses<T>>::contains_key(candidate.clone()), "A DAO can not be a member of other DAO");
            ensure!(<MembersCount>::get(dao_id) < Self::dao_maximum_number_of_members(dao_id), "Maximum number of members for this DAO is reached");
            ensure!(!<OpenDaoProposalsHashes<T>>::contains_key(proposal_hash), "This proposal already open");
            let len = open_proposals.len() as u32;
            ensure!(len < Self::open_proposals_per_block(), "Maximum number of open proposals is reached for the target block, try later");

            let dao_proposals_count = <DaoProposalsCount>::get(dao_id);
            let new_dao_proposals_count = dao_proposals_count
                .checked_add(1)
                .ok_or("Overflow adding a new DAO proposal")?;

            let proposal = Proposal {
                dao_id,
                action: Action::AddMember(candidate.clone()),
                open: true,
                accepted: false,
                voting_deadline,
                yes_count: 0,
                no_count: 0
            };
            let proposal_id = dao_proposals_count;
            open_proposals.push(proposal_id);

            <DaoProposals<T>>::insert((dao_id, proposal_id), proposal);
            <DaoProposalsCount>::insert(dao_id, new_dao_proposals_count);
            <DaoProposalsIndex>::insert(proposal_id, dao_id);
            <OpenDaoProposals<T>>::insert(voting_deadline, open_proposals);
            <OpenDaoProposalsHashes<T>>::insert(proposal_hash, proposal_id);
            <OpenDaoProposalsHashesIndex<T>>::insert(proposal_id, proposal_hash);

            Self::deposit_event(RawEvent::ProposeToAddMember(dao_id, candidate, voting_deadline));
            Ok(())
        }

        pub fn propose_to_remove_member(origin, dao_id: DaoId) -> DispatchResult {
            let candidate = ensure_signed(origin)?;

            let proposal_hash = ("propose_to_remove_member", &candidate, dao_id)
                .using_encoded(<T as system::Trait>::Hashing::hash);
            let voting_deadline = <system::Module<T>>::block_number() + <DaoTimeouts<T>>::get(dao_id);
            let mut open_proposals = Self::open_dao_proposals(voting_deadline);

            ensure!(<Daos<T>>::contains_key(dao_id), "This DAO not exists");
            ensure!(<DaoMembers<T>>::contains_key((dao_id, candidate.clone())), "You already are not a member of this DAO");
            ensure!(<MembersCount>::get(dao_id) > 1, "You are the last member of this DAO");
            ensure!(!<OpenDaoProposalsHashes<T>>::contains_key(proposal_hash), "This proposal already open");
            let len = open_proposals.len() as u32;
            ensure!(len < Self::open_proposals_per_block(), "Maximum number of open proposals is reached for the target block, try later");

            let dao_proposals_count = <DaoProposalsCount>::get(dao_id);
            let new_dao_proposals_count = dao_proposals_count
                .checked_add(1)
                .ok_or("Overflow adding a new DAO proposal")?;

            let proposal = Proposal {
                dao_id,
                action: Action::RemoveMember(candidate.clone()),
                open: true,
                accepted: false,
                voting_deadline,
                yes_count: 0,
                no_count: 0
            };
            let proposal_id = dao_proposals_count;
            open_proposals.push(proposal_id);

            <DaoProposals<T>>::insert((dao_id, proposal_id), proposal);
            <DaoProposalsCount>::insert(dao_id, new_dao_proposals_count);
            <DaoProposalsIndex>::insert(proposal_id, dao_id);
            <OpenDaoProposals<T>>::insert(voting_deadline, open_proposals);
            <OpenDaoProposalsHashes<T>>::insert(proposal_hash, proposal_id);
            <OpenDaoProposalsHashesIndex<T>>::insert(proposal_id, proposal_hash);

            Self::deposit_event(RawEvent::ProposeToRemoveMember(dao_id, candidate, voting_deadline));
            Ok(())
        }

        pub fn propose_to_get_loan(origin, dao_id: DaoId, description: Vec<u8>, days: Days, rate: Rate, token_id: TokenId, value: T::Balance) -> DispatchResult {
            let proposer = ensure_signed(origin)?;

            let proposal_hash = ("propose_to_get_loan", &proposer, dao_id, token_id)
                .using_encoded(<T as system::Trait>::Hashing::hash);
            let voting_deadline = <system::Module<T>>::block_number() + <DaoTimeouts<T>>::get(dao_id);
            let mut open_proposals = Self::open_dao_proposals(voting_deadline);

            Self::validate_description(&description)?;
            ensure!(<Daos<T>>::contains_key(dao_id), "This DAO not exists");
            ensure!(<DaoMembers<T>>::contains_key((dao_id, proposer.clone())), "You already are not a member of this DAO");
            ensure!(!<OpenDaoProposalsHashes<T>>::contains_key(proposal_hash), "This proposal already open");
            let len = open_proposals.len() as u32;
            ensure!(len < Self::open_proposals_per_block(), "Maximum number of open proposals is reached for the target block, try later");
            Self::withdraw_from_dao_balance_is_valid(dao_id, value)?;

            let dao_proposals_count = <DaoProposalsCount>::get(dao_id);
            let new_dao_proposals_count = dao_proposals_count
                .checked_add(1)
                .ok_or("Overflow adding a new DAO proposal")?;

            let proposal = Proposal {
                dao_id,
                action: Action::GetLoan(description, days, rate, token_id, value),
                open: true,
                accepted: false,
                voting_deadline,
                yes_count: 0,
                no_count: 0
            };
            let proposal_id = dao_proposals_count;
            open_proposals.push(proposal_id);

            <DaoProposals<T>>::insert((dao_id, proposal_id), proposal);
            <DaoProposalsCount>::insert(dao_id, new_dao_proposals_count);
            <DaoProposalsIndex>::insert(proposal_id, dao_id);
            <OpenDaoProposals<T>>::insert(voting_deadline, open_proposals);
            <OpenDaoProposalsHashes<T>>::insert(proposal_hash, proposal_id);
            <OpenDaoProposalsHashesIndex<T>>::insert(proposal_id, proposal_hash);

            Self::deposit_event(RawEvent::ProposeToGetLoan(dao_id, proposer, days, rate, value, voting_deadline));
            Ok(())
        }

        pub fn propose_to_change_vote_timeout(origin, dao_id: DaoId, value: T::BlockNumber) -> DispatchResult {
            let proposer = ensure_signed(origin)?;

            let proposal_hash = ("propose_to_change_vote_timeout", &proposer, dao_id)
                .using_encoded(<T as system::Trait>::Hashing::hash);
            let voting_deadline = <system::Module<T>>::block_number() + <DaoTimeouts<T>>::get(dao_id);
            let mut open_proposals = Self::open_dao_proposals(voting_deadline);

            Self::validate_vote_timeout(value)?;
            ensure!(<Daos<T>>::contains_key(dao_id), "This DAO not exists");
            ensure!(<DaoMembers<T>>::contains_key((dao_id, proposer.clone())), "You are not a member of this DAO");
            ensure!(!<OpenDaoProposalsHashes<T>>::contains_key(proposal_hash), "This proposal already open");
            ensure!(<DaoTimeouts<T>>::get(dao_id) != value, "new vote timeout equal current vote timeout");
            let len = open_proposals.len() as u32;
            ensure!(len < Self::open_proposals_per_block(), "Maximum number of open proposals is reached for the target block, try later");

            let dao_proposals_count = <DaoProposalsCount>::get(dao_id);
            let new_dao_proposals_count = dao_proposals_count
                .checked_add(1)
                .ok_or("Overflow adding a new DAO proposal")?;

            let proposal = Proposal {
                dao_id,
                action: Action::ChangeTimeout(dao_id, value),
                open: true,
                accepted: false,
                voting_deadline,
                yes_count: 0,
                no_count: 0
            };

            let proposal_id = dao_proposals_count;
            open_proposals.push(proposal_id);
            <DaoProposals<T>>::insert((dao_id, proposal_id), proposal);
            <DaoProposalsCount>::insert(dao_id, new_dao_proposals_count);
            <DaoProposalsIndex>::insert(proposal_id, dao_id);
            <OpenDaoProposals<T>>::insert(voting_deadline, open_proposals);
            <OpenDaoProposalsHashes<T>>::insert(proposal_hash, proposal_id);
            <OpenDaoProposalsHashesIndex<T>>::insert(proposal_id, proposal_hash);
            Self::deposit_event(RawEvent::ProposeToChangeTimeout(dao_id, value));
            Ok(())
        }

        pub fn propose_to_change_maximum_number_of_members(origin, dao_id: DaoId, value: MemberId) -> DispatchResult {
            let proposer = ensure_signed(origin)?;

            let proposal_hash = ("propose_to_change_maximum_number_of_members", &proposer, dao_id)
                .using_encoded(<T as system::Trait>::Hashing::hash);
            let voting_deadline = <system::Module<T>>::block_number() + <DaoTimeouts<T>>::get(dao_id);
            let mut open_proposals = Self::open_dao_proposals(voting_deadline);

            Self::validate_number_of_members(value)?;
            ensure!(<Daos<T>>::contains_key(dao_id), "This DAO not exists");
            ensure!(<DaoMembers<T>>::contains_key((dao_id, proposer.clone())), "You are not a member of this DAO");
            ensure!(!<OpenDaoProposalsHashes<T>>::contains_key(proposal_hash), "This proposal already open");
            ensure!(Self::dao_maximum_number_of_members(dao_id) != value, "New maximum number of members equal current number of members");
            ensure!(Self::members_count(dao_id) <= value, "The current number of members in this DAO more than the new maximum number of members");
            let len = open_proposals.len() as u32;
            ensure!(len < Self::open_proposals_per_block(), "Maximum number of open proposals is reached for the target block, try later");

            let dao_proposals_count = <DaoProposalsCount>::get(dao_id);
            let new_dao_proposals_count = dao_proposals_count
                .checked_add(1)
                .ok_or("Overflow adding a new DAO proposal")?;

            let proposal = Proposal {
                dao_id,
                action: Action::ChangeMaximumNumberOfMembers(dao_id, value),
                open: true,
                accepted: false,
                voting_deadline,
                yes_count: 0,
                no_count: 0
            };

            let proposal_id = dao_proposals_count;
            open_proposals.push(proposal_id);
            <DaoProposals<T>>::insert((dao_id, proposal_id), proposal);
            <DaoProposalsCount>::insert(dao_id, new_dao_proposals_count);
            <DaoProposalsIndex>::insert(proposal_id, dao_id);
            <OpenDaoProposals<T>>::insert(voting_deadline, open_proposals);
            <OpenDaoProposalsHashes<T>>::insert(proposal_hash, proposal_id);
            <OpenDaoProposalsHashesIndex<T>>::insert(proposal_id, proposal_hash);
            Self::deposit_event(RawEvent::ProposeToChangeMaximumNumberOfMembers(dao_id, value));
            Ok(())
        }

        pub fn vote(origin, dao_id: DaoId, proposal_id: ProposalId, vote: bool) -> DispatchResult {
            let voter = ensure_signed(origin)?;

            ensure!(<DaoMembers<T>>::contains_key((dao_id, voter.clone())), "You are not a member of this DAO");
            ensure!(<DaoProposals<T>>::contains_key((dao_id, proposal_id)), "This proposal not exists");
            ensure!(!<DaoProposalsVotesIndex<T>>::contains_key((dao_id, proposal_id, voter.clone())), "You voted already");

            let dao_proposal_votes_count = <DaoProposalsVotesCount>::get((dao_id, proposal_id));
            let new_dao_proposals_votes_count = dao_proposal_votes_count
                .checked_add(1)
                .ok_or("Overwlow adding a new vote of DAO proposal")?;

            let mut proposal = <DaoProposals<T>>::get((dao_id, proposal_id));
            ensure!(proposal.open, "This proposal is not open");

            if vote {
                proposal.yes_count += 1;
            } else {
                proposal.no_count += 1;
            }

            let dao_members_count = <MembersCount>::get(dao_id);
            let proposal_is_accepted = Self::votes_are_enough(proposal.yes_count, dao_members_count);
            let proposal_is_rejected = Self::votes_are_enough(proposal.no_count, dao_members_count);
            let all_member_voted = dao_members_count <= proposal.yes_count + proposal.no_count;

            if proposal_is_accepted {
                Self::execute_proposal(&proposal)?;
            }

            if proposal_is_accepted || proposal_is_rejected || all_member_voted {
                Self::close_proposal(dao_id, proposal_id, proposal.clone(), proposal_is_accepted);
            } else {
                <DaoProposals<T>>::insert((dao_id, proposal_id), proposal.clone());
            }

            <DaoProposalsVotes<T>>::insert((dao_id, proposal_id, dao_proposal_votes_count), &voter);
            <DaoProposalsVotesCount>::insert((dao_id, proposal_id), new_dao_proposals_votes_count);
            <DaoProposalsVotesIndex<T>>::insert((dao_id, proposal_id, voter.clone()), dao_proposal_votes_count);

            Self::deposit_event(RawEvent::NewVote(dao_id, proposal_id, voter, vote));

            match (proposal_is_accepted, proposal_is_rejected, all_member_voted) {
                (true, _, _) => Self::deposit_event(RawEvent::ProposalIsAccepted(dao_id, proposal_id)),
                (_, true, _) => Self::deposit_event(RawEvent::ProposalIsRejected(dao_id, proposal_id)),
                (_, _, true) => Self::deposit_event(RawEvent::ProposalIsRejected(dao_id, proposal_id)),
                (_, _, _) => ()
            }

            Ok(())
        }

        pub fn deposit(origin, dao_id: DaoId, value: T::Balance) -> DispatchResult {
            let depositor = ensure_signed(origin)?;

            ensure!(<Daos<T>>::contains_key(dao_id), "This DAO not exists");
            ensure!(<DaoMembers<T>>::contains_key((dao_id, depositor.clone())), "You are not a member of this DAO");
            ensure!(<balances::Module<T>>::free_balance(&depositor) > value, "Insufficient balance for deposit");

            let dao_address = <Address<T>>::get(dao_id);
            Self::remove_account_lock(&dao_address);
            <balances::Module<T> as Currency<_>>::transfer(&depositor, &dao_address, value, ExistenceRequirement::KeepAlive)?;
            Self::set_account_lock(&dao_address);

            Self::deposit_event(RawEvent::NewDeposit(depositor, dao_address, value));

            Ok(())
        }

        fn on_finalize() {
            let block_number = <system::Module<T>>::block_number();
            Self::open_dao_proposals(block_number)
                .iter()
                .for_each(|&proposal_id| {
                    let dao_id = <DaoProposalsIndex>::get(proposal_id);
                    let proposal = <DaoProposals<T>>::get((dao_id, proposal_id));

                    if proposal.open {
                        Self::close_proposal(dao_id, proposal_id, proposal, false);

                        Self::deposit_event(RawEvent::ProposalIsExpired(dao_id, proposal_id));
                    }
                });

            <OpenDaoProposals<T>>::remove(block_number);
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        Balance = <T as balances::Trait>::Balance,
        AccountId = <T as system::Trait>::AccountId,
        BlockNumber = <T as system::Trait>::BlockNumber,
    {
        NewDeposit(AccountId, AccountId, Balance),
        DaoCreated(AccountId, AccountId, Vec<u8>),
        NewVote(DaoId, ProposalId, AccountId, bool),
        ProposalIsAccepted(DaoId, ProposalId),
        ProposalIsExpired(DaoId, ProposalId),
        ProposalIsRejected(DaoId, ProposalId),
        ProposeToAddMember(DaoId, AccountId, BlockNumber),
        ProposeToRemoveMember(DaoId, AccountId, BlockNumber),
        ProposeToGetLoan(DaoId, AccountId, Days, Rate, Balance, BlockNumber),
        ProposeToChangeTimeout(DaoId, BlockNumber),
        ProposeToChangeMaximumNumberOfMembers(DaoId, MemberId),
    }
);

impl<T: Trait> Module<T> {
    fn validate_name(name: &[u8]) -> DispatchResult {
        if name.len() < 10 {
            return Err(DispatchError::Other("The name is very short"));
        }
        if name.len() > 255 {
            return Err(DispatchError::Other("The name is very long"));
        }

        let is_valid_char = |&c| {
            (c >= 97 && c <= 122) || // 'a' - 'z'
            (c >= 65 && c <= 90) ||  // 'A' - 'Z'
            (c >= 48 && c <= 57) ||  // '0' - '9'
            c == 45 || c == 95 // '-', '_'
        };
        if !(name.iter().all(is_valid_char)) {
            return Err(DispatchError::Other("The name has invalid chars"));
        }

        Ok(())
    }

    fn validate_description(description: &[u8]) -> DispatchResult {
        if description.len() < 10 {
            return Err(DispatchError::Other("The description is very short"));
        }
        if description.len() > 4096 {
            return Err(DispatchError::Other("The description is very long"));
        }

        Ok(())
    }
    fn validate_vote_timeout(timeout: T::BlockNumber) -> DispatchResult {
        if timeout < T::BlockNumber::from(MINIMUM_VOTE_TIOMEOUT) {
            return Err(DispatchError::Other(
                "The vote timeout must be not less 30 blocks",
            ));
        }
        if timeout > T::BlockNumber::from(MAXIMUM_VOTE_TIMEOUT) {
            return Err(DispatchError::Other(
                "The vote timeout must be not more 777600 blocks",
            ));
        }

        Ok(())
    }

    fn validate_number_of_members(number_of_members: MemberId) -> DispatchResult {
        if number_of_members < Self::minimum_number_of_members() {
            return Err(DispatchError::Other(
                "The new maximum number of members is very small",
            ));
        }
        if number_of_members > Self::maximum_number_of_members() {
            return Err(DispatchError::Other(
                "The new maximum number of members is very big",
            ));
        }

        Ok(())
    }

    fn add_member(dao_id: DaoId, member: T::AccountId) -> DispatchResult {
        ensure!(
            <MembersCount>::get(dao_id) < Self::dao_maximum_number_of_members(dao_id),
            "Maximum number of members for this DAO is reached"
        );

        let members_count = <MembersCount>::get(dao_id);
        let new_members_count = members_count
            .checked_add(1)
            .ok_or("Overflow adding a member to DAO")?;

        <Members<T>>::insert((dao_id, members_count), &member);
        <MembersCount>::insert(dao_id, new_members_count);
        <DaoMembers<T>>::insert((dao_id, member), members_count);

        Ok(())
    }

    fn remove_member(dao_id: DaoId, member: T::AccountId) -> DispatchResult {
        let members_count = <MembersCount>::get(dao_id);
        ensure!(
            <MembersCount>::get(dao_id) > 1,
            "Cannot remove last member of this DAO"
        );

        let new_members_count = members_count
            .checked_sub(1)
            .ok_or("Underflow removing a member from DAO")?;
        let max_member_id = new_members_count;

        let member_id = <DaoMembers<T>>::get((dao_id, member.clone()));

        if member_id != max_member_id {
            let latest_member = <Members<T>>::get((dao_id, max_member_id));
            <Members<T>>::insert((dao_id, member_id), &latest_member);
            <DaoMembers<T>>::insert((dao_id, latest_member), member_id);
        }
        <Members<T>>::remove((dao_id, max_member_id));
        <MembersCount>::insert(dao_id, new_members_count);
        <DaoMembers<T>>::remove((dao_id, member));

        Ok(())
    }

    fn propose_investment(
        dao_id: DaoId,
        description: Vec<u8>,
        days: Days,
        rate: Rate,
        token_id: TokenId,
        value: T::Balance,
    ) -> DispatchResult {
        let token = <token::Module<T>>::token_map(token_id);
        //TODO: take last price instead of average?..
        let price = <price_oracle::Module<T>>::aggregated_prices(token.symbol)
            .1
            .into();

        Self::mint_loan_tokens(dao_id, token_id, price, value)?;

        <marketplace::Module<T>>::propose_investment(
            dao_id,
            description,
            days,
            rate,
            token_id,
            price,
            value,
        )
    }

    fn mint_loan_tokens(
        dao_id: DaoId,
        token_id: TokenId,
        price: T::Balance,
        value: T::Balance,
    ) -> DispatchResult {
        let address = <Address<T>>::get(dao_id);
        let tokens_amount = value / price;
        <token::Module<T>>::_mint(token_id, address, tokens_amount)?;

        Ok(())
    }

    fn change_timeout(dao_id: DaoId, timeout: T::BlockNumber) -> DispatchResult {
        <DaoTimeouts<T>>::mutate(dao_id, |old_timeout| *old_timeout = timeout);

        Ok(())
    }

    fn change_maximum_number_of_members(
        dao_id: DaoId,
        number_of_members: MemberId,
    ) -> DispatchResult {
        <DaoMaximumNumberOfMembers>::mutate(dao_id, |old_number_of_members| {
            *old_number_of_members = number_of_members
        });

        Ok(())
    }

    fn withdraw_from_dao_balance_is_valid(dao_id: DaoId, value: T::Balance) -> DispatchResult {
        let dao_address = <Address<T>>::get(dao_id);
        let dao_balance = <balances::Module<T>>::free_balance(dao_address);
        let allowed_dao_balance = dao_balance
            .checked_sub(&<T as balances::Trait>::ExistentialDeposit::get())
            .ok_or("DAO balance is less than existential deposit")?;

        ensure!(allowed_dao_balance > value, "DAO balance is not sufficient");

        Ok(())
    }

    fn close_proposal(
        dao_id: DaoId,
        proposal_id: ProposalId,
        mut proposal: Proposal<DaoId, T::AccountId, T::Balance, T::BlockNumber, MemberId>,
        proposal_is_accepted: bool,
    ) {
        proposal.open = false;
        proposal.accepted = proposal_is_accepted;
        let proposal_hash = <OpenDaoProposalsHashesIndex<T>>::get(proposal_id);

        <DaoProposals<T>>::insert((dao_id, proposal_id), proposal);
        <OpenDaoProposalsHashes<T>>::remove(proposal_hash);
        <OpenDaoProposalsHashesIndex<T>>::remove(proposal_id);
    }

    fn votes_are_enough(votes: MemberId, maximum_votes: MemberId) -> bool {
        votes as f64 / maximum_votes as f64 >= 0.51
    }

    fn execute_proposal(
        proposal: &Proposal<DaoId, T::AccountId, T::Balance, T::BlockNumber, MemberId>,
    ) -> DispatchResult {
        match &proposal.action {
            Action::AddMember(member) => Self::add_member(proposal.dao_id, member.clone()),
            Action::RemoveMember(member) => Self::remove_member(proposal.dao_id, member.clone()),
            Action::GetLoan(description, days, rate, token, value) => Self::propose_investment(
                proposal.dao_id,
                description.to_vec(),
                *days,
                *rate,
                *token,
                *value,
            ),
            Action::ChangeTimeout(dao_id, value) => Self::change_timeout(*dao_id, *value),
            Action::ChangeMaximumNumberOfMembers(dao_id, value) => {
                Self::change_maximum_number_of_members(*dao_id, *value)
            }
            Action::EmptyAction => Ok(()),
        }
    }

    fn set_account_lock(who: &T::AccountId) {
        <balances::Module<T>>::set_lock(
            LOCK_NAME,
            &who,
            <balances::Module<T>>::free_balance(who),
            WithdrawReasons::all(),
        );
    }

    fn remove_account_lock(who: &T::AccountId) {
        <balances::Module<T>>::remove_lock(LOCK_NAME, who);
    }
}

/// tests for this module
///
/// future memo for how to construct balances pallet error
/// balances::Error::<Test, _>::InsufficientBalance
#[cfg(test)]
mod tests {
    use super::*;

    use crate::bridge;
    use frame_support::{
        assert_noop, assert_ok, impl_outer_dispatch, impl_outer_origin, parameter_types,
        traits::{Get, ReservableCurrency},
        weights::Weight,
    };
    use sp_core::{H160, H256};
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
        dao::DaoModule,
        price_oracle::PriceOracleModule,
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
    }

    impl balances::Trait for Test {
        type Balance = Balance;
        type DustRemoval = ();
        type Event = ();
        type ExistentialDeposit = ExistentialDeposit;
        type AccountStore = system::Module<Test>;
    }

    parameter_types! {
        pub const MinimumPeriod: u64 = 5;
    }
    impl timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = MinimumPeriod;
    }
    impl marketplace::Trait for Test {
        type Event = ();
    }
    impl token::Trait for Test {
        type Event = ();
    }
    impl bridge::Trait for Test {
        type Event = ();
    }

    pub type Extrinsic = TestXt<Call, ()>;
    type SubmitPFTransaction =
        system::offchain::TransactionSubmitter<price_oracle::crypto::Public, Call, Extrinsic>;

    parameter_types! {
        pub const BlockFetchPeriod: BlockNumber = 2;
        pub const GracePeriod: BlockNumber = 5;
    }

    impl price_oracle::Trait for Test {
        type Event = ();
        type Call = Call;
        type SubmitUnsignedTransaction = SubmitPFTransaction;

        // Wait period between automated fetches. Set to 0 disable this feature.
        //   Then you need to manucally kickoff pricefetch
        type GracePeriod = GracePeriod;
        type BlockFetchPeriod = BlockFetchPeriod;
    }

    impl Trait for Test {
        type Event = ();
    }
    type Balances = balances::Module<Test>;
    type BridgeModule = bridge::Module<Test>;
    type TokenModule = token::Module<Test>;
    type PriceOracleModule = price_oracle::Module<Test>;
    type DaoModule = Module<Test>;

    const DAO_ID: DaoId = 0;
    const DAO_NAME: &[u8; 10] = b"Name-1234_";
    const DAO_NAME2: &[u8; 10] = b"Name-5678_";
    const DAO_DESC: &[u8; 10] = b"Desc-1234_";
    const PROPOSAL_DESC: &[u8; 10] = b"Desc-5678_";
    const USER: u64 = 1;
    const USER2: u64 = 2;
    const USER3: u64 = 3;
    const USER4: u64 = 4;
    const USER5: u64 = 5;
    const V1: u64 = 1876;
    const V2: u64 = 2873;
    const V3: u64 = 3346;
    const EMPTY_USER: u64 = 6;
    const DAO: u64 = 11;
    const DAO2: u64 = 12;
    const NOT_EMPTY_DAO: u64 = 13;
    const NOT_EMPTY_DAO_BALANCE: u128 = 1000;
    const DAYS: Days = 365;
    const RATE: Rate = 1000;
    const VALUE: u128 = 10_000_000_000_000_000_000; // 10 unit tokens with 18 precision
    const VALUE2: u128 = 100_000_000_000_000_000_000; // 100 unit tokens with 18 precision
    const VOTE_TIMEOUT: u32 = MINIMUM_VOTE_TIOMEOUT + 1;
    const VERY_SMALL_VOTE_TIMEOUT: u32 = MINIMUM_VOTE_TIOMEOUT - 1;
    const VERY_BIG_VOTE_TIMEOUT: u32 = MAXIMUM_VOTE_TIMEOUT + 1;
    const TOKEN_ID: TokenId = 0;
    const PROPOSAL_ID: ProposalId = 0;
    const YES: bool = true;
    const NO: bool = false;
    const AMOUNT: u128 = 5000;
    const AMOUNT2: u128 = 100;
    const ADD_MEMBER1: ProposalId = 0;

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
                    (USER, VALUE2 + 100_000_000),
                    (V1, 100000),
                    (V2, 100000),
                    (V3, 100000),
                    (DAO, 500),
                    (NOT_EMPTY_DAO, NOT_EMPTY_DAO_BALANCE),
                    (USER3, 300_000),
                    (EMPTY_USER, 500),
                ],
            }
            .assimilate_storage(&mut storage);

            let _ = bridge::GenesisConfig::<Test> {
                validators_count: 3u32,
                validator_accounts: vec![V1, V2, V3],
                current_limits: vec![
                    100 * 10u128.pow(18),
                    200 * 10u128.pow(18),
                    50 * 10u128.pow(18),
                    400 * 10u128.pow(18),
                    10 * 10u128.pow(18),
                ],
            }
            .assimilate_storage(&mut storage);

            let ext = sp_io::TestExternalities::from(storage);
            ext
        }
    }

    /// KNOWN BUGS:
    ///     1. Tests can fail with assert_noop! bug: fails through different root hashes
    ///        looks like gibberish bytes:
    ///           left: `[165, 194, 103, 240, 170, 69, 230, 138, 137, 91, 252, 136, 82, 107, 223, 18, 184, 66, 180, 85, 190, 250, 56, 101, 20, 16, 197, 49, 183, 246, 12, 130]`,
    ///           right: `[60, 139, 20, 240, 52, 18, 65, 144, 55, 126, 157, 163, 147, 251, 22, 66, 21, 36, 34, 104, 183, 147, 220, 11, 145, 2, 1, 202, 170, 51, 82, 133]
    ///        solution: Write to storage after check - verify first, write last © shawntabrizi
    ///                 (or use assert_eq!(expr, Err("Error string")) explicitly)
    ///
    #[test]
    fn create_dao_should_work() {
        ExtBuilder::default().build().execute_with(|| {
            const MEMBER_ID: MemberId = 0;

            assert_eq!(DaoModule::daos_count(), 0);
            assert_eq!(DaoModule::members_count(DAO_ID), 0);
            assert_ne!(DaoModule::members((DAO_ID, MEMBER_ID)), USER);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);
            assert_eq!(DaoModule::members_count(DAO_ID), 1);
            assert_eq!(DaoModule::members((DAO_ID, MEMBER_ID)), USER);
            assert_eq!(DaoModule::dao_members((DAO_ID, USER)), MEMBER_ID);
        })
    }

    #[test]
    fn create_dao_case_founder_address_match_dao_address() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_noop!(
                DaoModule::create(
                    Origin::signed(USER),
                    USER,
                    DAO_NAME.to_vec(),
                    DAO_DESC.to_vec()
                ),
                "Founder address matches DAO address"
            );
            assert_eq!(DaoModule::daos_count(), 0);
        })
    }

    #[test]
    fn dao_name_is_very_short() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_noop!(
                DaoModule::create(
                    Origin::signed(USER),
                    DAO,
                    DAO_NAME.to_vec().drain(1..).collect(),
                    DAO_DESC.to_vec()
                ),
                "The name is very short"
            );
            assert_eq!(DaoModule::daos_count(), 0);
        })
    }

    #[test]
    fn dao_name_has_invalid_chars() {
        ExtBuilder::default().build().execute_with(|| {
            const ASCII_CODE_OF_PLUS: u8 = 43;

            let mut name = DAO_NAME.to_vec();
            name.push(ASCII_CODE_OF_PLUS);

            assert_eq!(DaoModule::daos_count(), 0);
            assert_noop!(
                DaoModule::create(Origin::signed(USER), DAO, name, DAO_DESC.to_vec()),
                "The name has invalid chars"
            );
            assert_eq!(DaoModule::daos_count(), 0);
        })
    }

    #[test]
    fn dao_name_is_very_long() {
        ExtBuilder::default().build().execute_with(|| {
            const ASCII_CODE_OF_A: u8 = 97;

            assert_eq!(DaoModule::daos_count(), 0);
            assert_noop!(
                DaoModule::create(
                    Origin::signed(USER),
                    DAO,
                    [ASCII_CODE_OF_A; 256].to_vec(),
                    DAO_DESC.to_vec()
                ),
                "The name is very long"
            );
            assert_eq!(DaoModule::daos_count(), 0);
        })
    }

    #[test]
    fn dao_description_is_very_short() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_noop!(
                DaoModule::create(
                    Origin::signed(USER),
                    DAO,
                    DAO_NAME.to_vec(),
                    DAO_DESC.to_vec().drain(1..).collect()
                ),
                "The description is very short"
            );
            assert_eq!(DaoModule::daos_count(), 0);
        })
    }

    #[test]
    fn dao_description_is_very_long() {
        ExtBuilder::default().build().execute_with(|| {
            const ASCII_CODE_OF_A: u8 = 97;

            assert_eq!(DaoModule::daos_count(), 0);
            assert_noop!(
                DaoModule::create(
                    Origin::signed(USER),
                    DAO,
                    DAO_NAME.to_vec().to_vec(),
                    [ASCII_CODE_OF_A; 4097].to_vec()
                ),
                "The description is very long"
            );
            assert_eq!(DaoModule::daos_count(), 0);
        })
    }

    #[test]
    fn dao_address_already_busy() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_noop!(
                DaoModule::create(
                    Origin::signed(USER),
                    DAO,
                    DAO_NAME.to_vec(),
                    DAO_DESC.to_vec()
                ),
                "This DAO address already busy"
            );
            assert_eq!(DaoModule::daos_count(), 1);
        })
    }

    #[test]
    fn dao_name_already_exists() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_noop!(
                DaoModule::create(
                    Origin::signed(USER),
                    DAO2,
                    DAO_NAME.to_vec(),
                    DAO_DESC.to_vec()
                ),
                "This DAO name already exists"
            );
            assert_eq!(DaoModule::daos_count(), 1);
        })
    }

    #[test]
    fn create_case_reserved_balance_is_not_0() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(Balances::reserve(&NOT_EMPTY_DAO, NOT_EMPTY_DAO_BALANCE));
            assert_noop!(
                DaoModule::create(
                    Origin::signed(USER),
                    NOT_EMPTY_DAO,
                    DAO_NAME.to_vec(),
                    DAO_DESC.to_vec()
                ),
                "Reserved balance of DAO address is not 0"
            );
            assert_eq!(DaoModule::daos_count(), 0);
        })
    }

    #[test]
    fn propose_to_add_member_should_work() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);
            assert_eq!(DaoModule::dao_proposals_count(DAO_ID), 0);
            assert_ok!(DaoModule::propose_to_add_member(
                Origin::signed(USER2),
                DAO_ID
            ));
            assert_eq!(DaoModule::dao_proposals_count(DAO_ID), 1);
        })
    }

    #[test]
    fn propose_to_add_member_case_this_dao_not_exists() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_noop!(
                DaoModule::propose_to_add_member(Origin::signed(USER), DAO_ID),
                "This DAO not exists"
            );
        })
    }

    #[test]
    fn propose_to_add_member_case_you_already_are_a_member_of_this_dao() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);
            assert_eq!(DaoModule::members((DAO_ID, 0)), USER);
            assert_noop!(
                DaoModule::propose_to_add_member(Origin::signed(USER), DAO_ID),
                "You already are a member of this DAO"
            );
        })
    }

    #[test]
    fn propose_to_add_member_case_dao_can_not_be_a_member_of_other_dao() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO2,
                DAO_NAME2.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 2);
            assert_eq!(DaoModule::members((DAO_ID, 0)), USER);
            assert_eq!(DaoModule::members((DAO_ID, 0)), USER);
            assert_noop!(
                DaoModule::propose_to_add_member(Origin::signed(DAO), DAO_ID),
                "A DAO can not be a member of other DAO"
            );
        })
    }

    #[test]
    fn propose_to_add_member_case_maximum_number_of_members_is_reached() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);
            assert_ok!(DaoModule::add_member(DAO_ID, USER2));
            assert_ok!(DaoModule::add_member(DAO_ID, USER3));
            assert_ok!(DaoModule::add_member(DAO_ID, USER4));
            assert_eq!(DaoModule::members_count(DAO_ID), 4);
            assert_noop!(
                DaoModule::propose_to_add_member(Origin::signed(USER5), DAO_ID),
                "Maximum number of members for this DAO is reached"
            );
        })
    }

    #[test]
    fn propose_to_add_member_case_this_proposal_already_open() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);
            assert_eq!(DaoModule::members((DAO_ID, 0)), USER);
            assert_ok!(DaoModule::propose_to_add_member(
                Origin::signed(USER2),
                DAO_ID
            ));
            assert_noop!(
                DaoModule::propose_to_add_member(Origin::signed(USER2), DAO_ID),
                "This proposal already open"
            );
        })
    }

    #[test]
    fn propose_to_add_member_case_maximum_number_of_open_proposals_is_reached() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);
            assert_eq!(DaoModule::members((DAO_ID, 0)), USER);
            assert_ok!(DaoModule::propose_to_add_member(
                Origin::signed(USER2),
                DAO_ID
            ));
            assert_ok!(DaoModule::propose_to_add_member(
                Origin::signed(USER3),
                DAO_ID
            ));
            assert_noop!(
                DaoModule::propose_to_add_member(Origin::signed(USER4), DAO_ID),
                "Maximum number of open proposals is reached for the target block, try later"
            );
        })
    }

    #[test]
    fn propose_to_remove_member_should_work() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);

            assert_ok!(DaoModule::add_member(DAO_ID, USER2));
            assert_eq!(DaoModule::dao_proposals_count(DAO_ID), 0);
            assert_ok!(DaoModule::propose_to_remove_member(
                Origin::signed(USER2),
                DAO_ID
            ));
            assert_eq!(DaoModule::dao_proposals_count(DAO_ID), 1);
        })
    }

    #[test]
    fn propose_to_remove_member_case_this_dao_not_exists() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_noop!(
                DaoModule::propose_to_remove_member(Origin::signed(USER), DAO_ID),
                "This DAO not exists"
            );
        })
    }

    #[test]
    fn propose_to_remove_member_case_you_already_are_not_member() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);
            assert_noop!(
                DaoModule::propose_to_remove_member(Origin::signed(USER2), DAO_ID),
                "You already are not a member of this DAO"
            );
        })
    }

    #[test]
    fn propose_to_remove_member_case_you_are_the_latest_member() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);
            assert_noop!(
                DaoModule::propose_to_remove_member(Origin::signed(USER), DAO_ID),
                "You are the last member of this DAO"
            );
        })
    }

    #[test]
    fn propose_to_remove_member_case_this_proposal_already_open() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);
            assert_eq!(DaoModule::members((DAO_ID, 0)), USER);
            assert_ok!(DaoModule::add_member(DAO_ID, USER2));
            assert_ok!(DaoModule::propose_to_remove_member(
                Origin::signed(USER2),
                DAO_ID
            ));
            assert_noop!(
                DaoModule::propose_to_remove_member(Origin::signed(USER2), DAO_ID),
                "This proposal already open"
            );
        })
    }

    #[test]
    fn propose_to_remove_member_case_maximum_number_of_open_proposals_is_reached() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);
            assert_eq!(DaoModule::members((DAO_ID, 0)), USER);
            assert_ok!(DaoModule::add_member(DAO_ID, USER2));
            assert_ok!(DaoModule::add_member(DAO_ID, USER3));
            assert_ok!(DaoModule::add_member(DAO_ID, USER4));
            assert_ok!(DaoModule::propose_to_remove_member(
                Origin::signed(USER2),
                DAO_ID
            ));
            assert_ok!(DaoModule::propose_to_remove_member(
                Origin::signed(USER3),
                DAO_ID
            ));
            assert_noop!(
                DaoModule::propose_to_remove_member(Origin::signed(USER4), DAO_ID),
                "Maximum number of open proposals is reached for the target block, try later"
            );
        })
    }

    #[test]
    fn propose_to_get_loan_case_maximum_number_of_open_proposals_is_reached() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);
            assert_eq!(DaoModule::members((DAO_ID, 0)), USER);
            assert_ok!(DaoModule::add_member(DAO_ID, USER2));
            assert_ok!(DaoModule::add_member(DAO_ID, USER3));
            assert_ok!(DaoModule::propose_to_get_loan(
                Origin::signed(USER),
                DAO_ID,
                PROPOSAL_DESC.to_vec(),
                DAYS,
                RATE,
                TOKEN_ID,
                AMOUNT2
            ));
            assert_ok!(DaoModule::propose_to_get_loan(
                Origin::signed(USER2),
                DAO_ID,
                PROPOSAL_DESC.to_vec(),
                DAYS,
                RATE,
                TOKEN_ID,
                AMOUNT2
            ));
            assert_noop!(
                DaoModule::propose_to_get_loan(
                    Origin::signed(USER3),
                    DAO_ID,
                    PROPOSAL_DESC.to_vec(),
                    DAYS,
                    RATE,
                    TOKEN_ID,
                    AMOUNT2
                ),
                "Maximum number of open proposals is reached for the target block, try later"
            );
        })
    }

    #[test]
    fn propose_to_get_loan_should_work() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);

            assert_eq!(DaoModule::dao_proposals_count(DAO_ID), 0);
            assert_ok!(DaoModule::propose_to_get_loan(
                Origin::signed(USER),
                DAO_ID,
                PROPOSAL_DESC.to_vec(),
                DAYS,
                RATE,
                TOKEN_ID,
                AMOUNT2
            ));
            assert_eq!(DaoModule::dao_proposals_count(DAO_ID), 1);
        })
    }

    #[test]
    fn propose_to_get_loan_case_this_dao_not_exists() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_noop!(
                DaoModule::propose_to_get_loan(
                    Origin::signed(USER),
                    DAO_ID,
                    PROPOSAL_DESC.to_vec(),
                    DAYS,
                    RATE,
                    TOKEN_ID,
                    AMOUNT2
                ),
                "This DAO not exists"
            );
        })
    }

    #[test]
    fn propose_to_get_loan_case_you_already_are_not_member() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);
            assert_noop!(
                DaoModule::propose_to_get_loan(
                    Origin::signed(USER2),
                    DAO_ID,
                    PROPOSAL_DESC.to_vec(),
                    DAYS,
                    RATE,
                    TOKEN_ID,
                    AMOUNT2
                ),
                "You already are not a member of this DAO"
            );
        })
    }

    #[test]
    fn propose_to_get_loan_case_this_proposal_already_open() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);
            assert_eq!(Balances::free_balance(DAO), 1000);
            assert_eq!(DaoModule::members((DAO_ID, 0)), USER);
            assert_ok!(DaoModule::add_member(DAO_ID, USER2));
            assert_ok!(DaoModule::propose_to_get_loan(
                Origin::signed(USER),
                DAO_ID,
                PROPOSAL_DESC.to_vec(),
                DAYS,
                RATE,
                TOKEN_ID,
                AMOUNT2
            ));
            assert_noop!(
                DaoModule::propose_to_get_loan(
                    Origin::signed(USER),
                    DAO_ID,
                    PROPOSAL_DESC.to_vec(),
                    DAYS,
                    RATE,
                    TOKEN_ID,
                    AMOUNT2
                ),
                "This proposal already open"
            );
        })
    }

    #[test]
    fn vote_should_work() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);
            assert_ok!(DaoModule::propose_to_add_member(
                Origin::signed(USER2),
                DAO_ID
            ));
            assert_eq!(
                DaoModule::dao_proposals_votes_count((DAO_ID, PROPOSAL_ID)),
                0
            );
            assert_ok!(DaoModule::vote(
                Origin::signed(USER),
                DAO_ID,
                PROPOSAL_ID,
                YES
            ));
            assert_eq!(
                DaoModule::dao_proposals_votes_count((DAO_ID, PROPOSAL_ID)),
                1
            );
            assert_eq!(DaoModule::dao_proposals((DAO_ID, 0)).accepted, true)
        })
    }

    #[test]
    fn vote_should_work_early_ending_of_voting_case_all_yes() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);
            assert_ok!(DaoModule::add_member(DAO_ID, USER2));
            assert_ok!(DaoModule::add_member(DAO_ID, USER3));

            assert_eq!(DaoModule::members_count(DAO_ID), 3);
            assert_eq!(DaoModule::members((DAO_ID, 0)), USER);
            assert_eq!(DaoModule::members((DAO_ID, 1)), USER2);
            assert_eq!(DaoModule::members((DAO_ID, 2)), USER3);
            assert_ne!(DaoModule::members((DAO_ID, 3)), USER4);

            assert_ok!(DaoModule::propose_to_add_member(
                Origin::signed(USER4),
                DAO_ID
            ));

            assert_ok!(DaoModule::vote(
                Origin::signed(USER),
                DAO_ID,
                PROPOSAL_ID,
                YES
            ));
            assert_eq!(DaoModule::members_count(DAO_ID), 3);

            assert_eq!(DaoModule::dao_proposals((DAO_ID, 0)).open, true);
            assert_ok!(DaoModule::vote(
                Origin::signed(USER2),
                DAO_ID,
                PROPOSAL_ID,
                YES
            ));
            assert_eq!(DaoModule::dao_proposals((DAO_ID, 0)).open, false);
            assert_eq!(DaoModule::dao_proposals((DAO_ID, 0)).accepted, true);
            assert_eq!(DaoModule::members_count(DAO_ID), 4);

            assert_noop!(
                DaoModule::vote(Origin::signed(USER3), DAO_ID, PROPOSAL_ID, YES),
                "This proposal is not open"
            );

            assert_eq!(DaoModule::members_count(DAO_ID), 4);
            assert_eq!(DaoModule::members((DAO_ID, 0)), USER);
            assert_eq!(DaoModule::members((DAO_ID, 1)), USER2);
            assert_eq!(DaoModule::members((DAO_ID, 2)), USER3);
            assert_eq!(DaoModule::members((DAO_ID, 3)), USER4);
        })
    }

    #[test]
    fn vote_should_work_early_ending_of_voting_case_all_no() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);
            assert_ok!(DaoModule::add_member(DAO_ID, USER2));
            assert_ok!(DaoModule::add_member(DAO_ID, USER3));

            assert_eq!(DaoModule::members_count(DAO_ID), 3);
            assert_eq!(DaoModule::members((DAO_ID, 0)), USER);
            assert_eq!(DaoModule::members((DAO_ID, 1)), USER2);
            assert_eq!(DaoModule::members((DAO_ID, 2)), USER3);
            assert_ne!(DaoModule::members((DAO_ID, 3)), USER4);

            assert_ok!(DaoModule::propose_to_add_member(
                Origin::signed(USER4),
                DAO_ID
            ));

            assert_ok!(DaoModule::vote(
                Origin::signed(USER),
                DAO_ID,
                PROPOSAL_ID,
                NO
            ));
            assert_eq!(DaoModule::members_count(DAO_ID), 3);

            assert_eq!(DaoModule::dao_proposals((DAO_ID, 0)).open, true);
            assert_ok!(DaoModule::vote(
                Origin::signed(USER2),
                DAO_ID,
                PROPOSAL_ID,
                NO
            ));
            assert_eq!(DaoModule::dao_proposals((DAO_ID, 0)).open, false);
            assert_eq!(DaoModule::dao_proposals((DAO_ID, 0)).accepted, false);
            assert_eq!(DaoModule::members_count(DAO_ID), 3);

            assert_noop!(
                DaoModule::vote(Origin::signed(USER3), DAO_ID, PROPOSAL_ID, NO),
                "This proposal is not open"
            );

            assert_eq!(DaoModule::members_count(DAO_ID), 3);
            assert_eq!(DaoModule::members((DAO_ID, 0)), USER);
            assert_eq!(DaoModule::members((DAO_ID, 1)), USER2);
            assert_eq!(DaoModule::members((DAO_ID, 2)), USER3);
            assert_ne!(DaoModule::members((DAO_ID, 3)), USER4);
        })
    }

    #[test]
    fn vote_should_work_early_ending_of_voting_case_all_members_voted() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);
            assert_ok!(DaoModule::add_member(DAO_ID, USER2));
            assert_ok!(DaoModule::add_member(DAO_ID, USER3));

            assert_eq!(DaoModule::members_count(DAO_ID), 3);
            assert_eq!(DaoModule::members((DAO_ID, 0)), USER);
            assert_eq!(DaoModule::members((DAO_ID, 1)), USER2);
            assert_eq!(DaoModule::members((DAO_ID, 2)), USER3);
            assert_ne!(DaoModule::members((DAO_ID, 3)), USER4);

            assert_ok!(DaoModule::propose_to_add_member(
                Origin::signed(USER4),
                DAO_ID
            ));

            assert_ok!(DaoModule::vote(
                Origin::signed(USER),
                DAO_ID,
                PROPOSAL_ID,
                YES
            ));
            assert_eq!(DaoModule::members_count(DAO_ID), 3);

            assert_ok!(DaoModule::vote(
                Origin::signed(USER2),
                DAO_ID,
                PROPOSAL_ID,
                NO
            ));
            assert_eq!(DaoModule::members_count(DAO_ID), 3);

            assert_eq!(DaoModule::dao_proposals((DAO_ID, 0)).open, true);
            assert_ok!(DaoModule::vote(
                Origin::signed(USER3),
                DAO_ID,
                PROPOSAL_ID,
                NO
            ));
            assert_eq!(DaoModule::dao_proposals((DAO_ID, 0)).open, false);
            assert_eq!(DaoModule::dao_proposals((DAO_ID, 0)).accepted, false);

            assert_eq!(DaoModule::members_count(DAO_ID), 3);
            assert_eq!(DaoModule::members((DAO_ID, 0)), USER);
            assert_eq!(DaoModule::members((DAO_ID, 1)), USER2);
            assert_eq!(DaoModule::members((DAO_ID, 2)), USER3);
            assert_ne!(DaoModule::members((DAO_ID, 3)), USER4);
        })
    }

    #[test]
    fn vote_case_you_are_not_member_of_this_dao() {
        ExtBuilder::default().build().execute_with(|| {

            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);
            assert_noop!(
                DaoModule::vote(Origin::signed(USER2), DAO_ID, PROPOSAL_ID, YES),
                "You are not a member of this DAO"
            );
        })
    }

    #[test]
    fn vote_case_this_proposal_not_exists() {
        ExtBuilder::default().build().execute_with(|| {

            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);
            assert_noop!(
                DaoModule::vote(Origin::signed(USER), DAO_ID, PROPOSAL_ID, YES),
                "This proposal not exists"
            );
        })
    }

    #[test]
    fn vote_case_you_voted_already() {
        ExtBuilder::default().build().execute_with(|| {

            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);
            assert_ok!(DaoModule::propose_to_add_member(
                Origin::signed(USER2),
                DAO_ID
            ));
            assert_ok!(DaoModule::vote(
                Origin::signed(USER),
                DAO_ID,
                PROPOSAL_ID,
                YES
            ));
            assert_noop!(
                DaoModule::vote(Origin::signed(USER), DAO_ID, PROPOSAL_ID, YES),
                "You voted already"
            );
        })
    }

    #[test]
    fn vote_case_this_proposal_is_not_open() {
        ExtBuilder::default().build().execute_with(|| {

            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);
            assert_ok!(DaoModule::propose_to_add_member(
                Origin::signed(USER2),
                DAO_ID
            ));
            DaoModule::close_proposal(
                DAO_ID,
                PROPOSAL_ID,
                DaoModule::dao_proposals((DAO_ID, PROPOSAL_ID)),
                false,
            );
            assert_noop!(
                DaoModule::vote(Origin::signed(USER), DAO_ID, PROPOSAL_ID, YES),
                "This proposal is not open"
            );
        })
    }

    #[test]
    fn vote_case_maximum_number_of_members_is_reached() {
        ExtBuilder::default().build().execute_with(|| {

            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);

            assert_ok!(DaoModule::add_member(DAO_ID, USER2));
            assert_ok!(DaoModule::add_member(DAO_ID, USER3));
            assert_eq!(DaoModule::members_count(DAO_ID), 3);
            assert_ok!(DaoModule::propose_to_add_member(
                Origin::signed(USER4),
                DAO_ID
            ));
            assert_ok!(DaoModule::add_member(DAO_ID, USER5));
            assert_eq!(DaoModule::members_count(DAO_ID), 4);
            assert_ok!(DaoModule::vote(
                Origin::signed(USER),
                DAO_ID,
                PROPOSAL_ID,
                YES
            ));
            assert_ok!(DaoModule::vote(
                Origin::signed(USER2),
                DAO_ID,
                PROPOSAL_ID,
                YES
            ));
            assert_noop!(
                DaoModule::vote(Origin::signed(USER3), DAO_ID, PROPOSAL_ID, YES),
                "Maximum number of members for this DAO is reached"
            );
        })
    }

    #[test]
    fn deposit_should_work() {
        ExtBuilder::default().build().execute_with(|| {

            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            let dao_id = DaoModule::dao_addresses(DAO);
            assert_eq!(DaoModule::daos_count(), 1);
            assert_eq!(Balances::free_balance(DAO), 1000);

            assert_ok!(DaoModule::deposit(Origin::signed(USER), dao_id, AMOUNT));

            assert_eq!(Balances::free_balance(DAO), 6000);
        })
    }

    #[test]
    fn deposit_should_fail_not_enough() {
        ExtBuilder::default().build().execute_with(|| {

            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            let dao_id = DaoModule::dao_addresses(DAO);

            assert_eq!(DaoModule::daos_count(), 1);

            assert_ok!(DaoModule::propose_to_add_member(
                Origin::signed(EMPTY_USER),
                dao_id
            ));
            assert_ok!(DaoModule::vote(
                Origin::signed(USER),
                dao_id,
                PROPOSAL_ID,
                YES
            ));
            assert_eq!(DaoModule::members_count(dao_id), 2);

            assert_eq!(Balances::free_balance(DAO), 1000);

            assert_noop!(
                DaoModule::deposit(Origin::signed(EMPTY_USER), dao_id, AMOUNT),
                "Insufficient balance for deposit"
            );
        })
    }

    #[test]
    fn change_vote_timeout_should_work() {
        ExtBuilder::default().build().execute_with(|| {
            const CHANGE_TIMEOUT: ProposalId = 1;
            const CHANGE_TIMEOUT2: ProposalId = 2;

            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            let dao_id = DaoModule::dao_addresses(DAO);

            assert_eq!(Balances::free_balance(DAO), 1000);
            assert_eq!(DaoModule::daos_count(), 1);

            assert_ok!(DaoModule::propose_to_add_member(
                Origin::signed(USER2),
                dao_id
            ));
            assert_ok!(DaoModule::vote(
                Origin::signed(USER),
                dao_id,
                ADD_MEMBER1,
                YES
            ));
            assert_eq!(DaoModule::members_count(dao_id), 2);

            assert_ok!(DaoModule::deposit(Origin::signed(USER), dao_id, AMOUNT));
            assert_eq!(Balances::free_balance(DAO), 6000);
            let old_vote_timeout = DaoModule::dao_timeouts(dao_id);
            assert_ok!(DaoModule::propose_to_change_vote_timeout(
                Origin::signed(USER2),
                dao_id,
                VOTE_TIMEOUT.into()
            ));
            assert_ok!(DaoModule::vote(
                Origin::signed(USER),
                dao_id,
                CHANGE_TIMEOUT,
                YES
            ));
            assert_ok!(DaoModule::vote(
                Origin::signed(USER2),
                dao_id,
                CHANGE_TIMEOUT,
                YES
            ));

            let new_vote_timeout = DaoModule::dao_timeouts(dao_id);
            assert_ne!(new_vote_timeout, old_vote_timeout);

            assert_ok!(DaoModule::propose_to_change_vote_timeout(
                Origin::signed(USER2),
                dao_id,
                MINIMUM_VOTE_TIOMEOUT.into()
            ));
            assert_ok!(DaoModule::vote(
                Origin::signed(USER),
                dao_id,
                CHANGE_TIMEOUT2,
                YES
            ));
            assert_ok!(DaoModule::vote(
                Origin::signed(USER2),
                dao_id,
                CHANGE_TIMEOUT2,
                YES
            ));
            assert_eq!(DaoModule::dao_timeouts(dao_id), old_vote_timeout);
        })
    }

    #[test]
    fn change_vote_timeout_case_new_vote_timeout_equal_current_vote_timeout() {
        ExtBuilder::default().build().execute_with(|| {

            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            let dao_id = DaoModule::dao_addresses(DAO);

            assert_eq!(Balances::free_balance(DAO), 1000);
            assert_eq!(DaoModule::daos_count(), 1);

            assert_ok!(DaoModule::propose_to_add_member(
                Origin::signed(USER2),
                dao_id
            ));
            assert_ok!(DaoModule::vote(
                Origin::signed(USER),
                dao_id,
                ADD_MEMBER1,
                YES
            ));
            assert_eq!(DaoModule::members_count(dao_id), 2);

            assert_ok!(DaoModule::deposit(Origin::signed(USER), dao_id, AMOUNT));
            assert_eq!(Balances::free_balance(DAO), 6000);

            assert_noop!(
                DaoModule::propose_to_change_vote_timeout(
                    Origin::signed(USER2),
                    dao_id,
                    MINIMUM_VOTE_TIOMEOUT.into()
                ),
                "new vote timeout equal current vote timeout"
            );
        })
    }

    #[test]
    fn change_vote_timeout_case_new_voting_timeout_is_very_small() {
        ExtBuilder::default().build().execute_with(|| {

            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            let dao_id = DaoModule::dao_addresses(DAO);

            assert_eq!(Balances::free_balance(DAO), 1000);
            assert_eq!(DaoModule::daos_count(), 1);

            assert_ok!(DaoModule::propose_to_add_member(
                Origin::signed(USER2),
                dao_id
            ));
            assert_ok!(DaoModule::vote(
                Origin::signed(USER),
                dao_id,
                ADD_MEMBER1,
                YES
            ));
            assert_eq!(DaoModule::members_count(dao_id), 2);

            assert_ok!(DaoModule::deposit(Origin::signed(USER), dao_id, AMOUNT));
            assert_eq!(Balances::free_balance(DAO), 6000);
            assert_noop!(
                DaoModule::propose_to_change_vote_timeout(
                    Origin::signed(USER2),
                    dao_id,
                    VERY_SMALL_VOTE_TIMEOUT.into()
                ),
                "The vote timeout must be not less 30 blocks"
            );
        })
    }

    #[test]
    fn change_vote_timeout_case_new_voting_timeout_is_very_big() {
        ExtBuilder::default().build().execute_with(|| {

            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            let dao_id = DaoModule::dao_addresses(DAO);

            assert_eq!(Balances::free_balance(DAO), 1000);
            assert_eq!(DaoModule::daos_count(), 1);

            assert_ok!(DaoModule::propose_to_add_member(
                Origin::signed(USER2),
                dao_id
            ));
            assert_ok!(DaoModule::vote(
                Origin::signed(USER),
                dao_id,
                ADD_MEMBER1,
                YES
            ));
            assert_eq!(DaoModule::members_count(dao_id), 2);

            assert_ok!(DaoModule::deposit(Origin::signed(USER), dao_id, AMOUNT));
            assert_eq!(Balances::free_balance(DAO), 6000);
            assert_noop!(
                DaoModule::propose_to_change_vote_timeout(
                    Origin::signed(USER2),
                    dao_id,
                    VERY_BIG_VOTE_TIMEOUT.into()
                ),
                "The vote timeout must be not more 777600 blocks"
            );
        })
    }

    #[test]
    fn change_maximum_number_of_members_should_work() {
        ExtBuilder::default().build().execute_with(|| {
            const PROPOSAL_ID2: ProposalId = 1;

            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(Balances::free_balance(DAO), 1000);
            assert_eq!(DaoModule::daos_count(), 1);

            let dao_id = DaoModule::dao_addresses(DAO);

            let old_maximum_number_of_members = DaoModule::dao_maximum_number_of_members(dao_id);
            assert_ok!(DaoModule::propose_to_change_maximum_number_of_members(
                Origin::signed(USER),
                dao_id,
                DaoModule::minimum_number_of_members(),
            ));
            assert_ok!(DaoModule::vote(
                Origin::signed(USER),
                dao_id,
                PROPOSAL_ID,
                YES
            ));

            let new_maximum_number_of_members = DaoModule::dao_maximum_number_of_members(dao_id);
            assert_ne!(new_maximum_number_of_members, old_maximum_number_of_members);

            assert_ok!(DaoModule::propose_to_change_maximum_number_of_members(
                Origin::signed(USER),
                dao_id,
                DaoModule::maximum_number_of_members()
            ));
            assert_ok!(DaoModule::vote(
                Origin::signed(USER),
                dao_id,
                PROPOSAL_ID2,
                YES
            ));
            assert_eq!(
                DaoModule::dao_maximum_number_of_members(dao_id),
                DaoModule::maximum_number_of_members()
            );
        })
    }

    #[test]
    fn change_maximum_number_of_members_case_current_number_of_members_more_than_new_maximum_number_of_members(
    ) {
        ExtBuilder::default().build().execute_with( || {

            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(Balances::free_balance(DAO), 1000);
            assert_eq!(DaoModule::daos_count(), 1);

            let dao_id = DaoModule::dao_addresses(DAO);

            assert_ok!(DaoModule::propose_to_add_member(
                Origin::signed(USER2),
                dao_id
            ));
            assert_ok!(DaoModule::vote(
                Origin::signed(USER),
                dao_id,
                PROPOSAL_ID,
                YES
            ));
            assert_eq!(DaoModule::members_count(dao_id), 2);

            assert_noop!(DaoModule::propose_to_change_maximum_number_of_members(
                Origin::signed(USER),
                dao_id,
                DaoModule::members_count(dao_id) - 1,
            ), "The current number of members in this DAO more than the new maximum number of members");
        })
    }

    #[test]
    fn change_maximum_number_of_members_case_new_maximum_number_of_members_equal_current_maximum_number_of_members(
    ) {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(Balances::free_balance(DAO), 1000);
            assert_eq!(DaoModule::daos_count(), 1);

            let dao_id = DaoModule::dao_addresses(DAO);
            assert_noop!(
                DaoModule::propose_to_change_maximum_number_of_members(
                    Origin::signed(USER),
                    dao_id,
                    DaoModule::dao_maximum_number_of_members(dao_id),
                ),
                "New maximum number of members equal current number of members"
            );
        })
    }

    #[test]
    fn change_maximum_number_of_members_case_new_maximum_number_of_members_is_very_small() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(Balances::free_balance(DAO), 1000);
            assert_eq!(DaoModule::daos_count(), 1);

            assert_noop!(
                DaoModule::propose_to_change_maximum_number_of_members(
                    Origin::signed(USER2),
                    DaoModule::dao_addresses(DAO),
                    DaoModule::minimum_number_of_members() - 1,
                ),
                "The new maximum number of members is very small"
            );
        })
    }

    #[test]
    fn change_maximum_number_of_members_case_new_voting_timeout_is_very_big() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));

            assert_eq!(Balances::free_balance(DAO), 1000);
            assert_eq!(DaoModule::daos_count(), 1);

            assert_noop!(
                DaoModule::propose_to_change_maximum_number_of_members(
                    Origin::signed(USER),
                    DaoModule::dao_addresses(DAO),
                    DaoModule::maximum_number_of_members() + 1,
                ),
                "The new maximum number of members is very big"
            );
        })
    }

    #[test]
    fn remove_member_should_work() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            assert_eq!(DaoModule::daos_count(), 1);
            assert_eq!(DaoModule::members((DAO_ID, 0)), USER);

            assert_ok!(DaoModule::add_member(DAO_ID, USER2));
            assert_ok!(DaoModule::add_member(DAO_ID, USER3));
            assert_ok!(DaoModule::add_member(DAO_ID, USER4));
            assert_eq!(DaoModule::members((DAO_ID, 0)), USER);
            assert_eq!(DaoModule::members((DAO_ID, 1)), USER2);
            assert_eq!(DaoModule::members((DAO_ID, 2)), USER3);
            assert_eq!(DaoModule::members((DAO_ID, 3)), USER4);
            assert_eq!(DaoModule::members_count(DAO_ID), 4);

            assert_ok!(DaoModule::remove_member(DAO_ID, USER2));
            assert_eq!(DaoModule::members((DAO_ID, 0)), USER);
            assert_eq!(DaoModule::members((DAO_ID, 1)), USER4);
            assert_eq!(DaoModule::members((DAO_ID, 2)), USER3);
            assert_eq!(DaoModule::members_count(DAO_ID), 3);

            assert_ok!(DaoModule::remove_member(DAO_ID, USER));
            assert_eq!(DaoModule::members((DAO_ID, 0)), USER3);
            assert_eq!(DaoModule::members((DAO_ID, 1)), USER4);
            assert_eq!(DaoModule::members_count(DAO_ID), 2);

            assert_ok!(DaoModule::remove_member(DAO_ID, USER4));
            assert_eq!(DaoModule::members((DAO_ID, 0)), USER3);
            assert_eq!(DaoModule::members_count(DAO_ID), 1);
        })
    }

    #[test]
    fn withdraw_case_direct_withdraw_forbidden() {
        ExtBuilder::default().build().execute_with(|| {
            const AMOUNT2: u128 = AMOUNT - 1000;

            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));
            let dao_id = DaoModule::dao_addresses(DAO);

            assert_eq!(Balances::free_balance(DAO), 1000);
            assert_eq!(DaoModule::daos_count(), 1);
            assert_ok!(DaoModule::deposit(Origin::signed(USER), dao_id, AMOUNT));
            assert_eq!(Balances::free_balance(DAO), 6000);

            assert_noop!(
                Balances::transfer(Origin::signed(DAO), USER, AMOUNT2),
                balances::Error::<Test, _>::LiquidityRestrictions
            );
            assert_eq!(Balances::free_balance(DAO), 6000);
        })
    }

    #[ignore]
    #[test]
    fn withdraw_should_work() {
        ExtBuilder::default().build().execute_with(|| {
            const GET_LOAN: ProposalId = 1;
            const ETH_ADDRESS: &[u8; 20] = b"0x00b46c2526ebb8f4c9";
            
            let min_limit = 10 * 10u128.pow(18);
            let value = 15 * 10u128.pow(18);
            assert_eq!(BridgeModule::current_limits().min_tx_value, min_limit);

            // create dao
            assert_eq!(DaoModule::daos_count(), 0);
            assert_ok!(DaoModule::create(
                Origin::signed(USER),
                DAO,
                DAO_NAME.to_vec(),
                DAO_DESC.to_vec()
            ));

            assert_eq!(Balances::free_balance(DAO), 1000);
            assert_eq!(DaoModule::daos_count(), 1);

            // add someone
            assert_ok!(DaoModule::propose_to_add_member(
                Origin::signed(USER2),
                DAO_ID
            ));
            assert_ok!(DaoModule::vote(
                Origin::signed(USER),
                DAO_ID,
                ADD_MEMBER1,
                YES
            ));
            assert_eq!(DaoModule::members_count(DAO_ID), 2);
            // deposit some amount
            assert_ok!(DaoModule::deposit(Origin::signed(USER), DAO_ID, value));
            assert_eq!(Balances::free_balance(DAO), value + 1000);

            // create loan proposal
            assert_eq!(DaoModule::dao_proposals_count(DAO_ID), 1);
            assert_ok!(DaoModule::propose_to_get_loan(
                Origin::signed(USER),
                DAO_ID,
                PROPOSAL_DESC.to_vec(),
                DAYS,
                RATE,
                TOKEN_ID,
                value
            ));
            assert_eq!(DaoModule::dao_proposals_count(DAO_ID), 2);
            assert_ok!(DaoModule::vote(
                Origin::signed(USER2),
                DAO_ID,
                GET_LOAN,
                YES
            ));
            let token_amount = TokenModule::balance_of((TOKEN_ID, USER));

            // withdraw
            let eth_address = H160::from(ETH_ADDRESS);
            let amount1 = 600;

            // substrate ----> ETH
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER2),
                eth_address,
                TOKEN_ID,
                token_amount
            ));
            // RelayMessage(message_id) event emitted

            let sub_message_id = BridgeModule::message_id_by_transfer_id(0);
            let get_message = || BridgeModule::messages(sub_message_id);

            let mut message = get_message();
            assert_eq!(message.status, Status::Withdraw);

            // approval
            assert_eq!(TokenModule::locked((0, USER2)), 0);
            assert_ok!(BridgeModule::approve_transfer(
                Origin::signed(V1),
                sub_message_id
            ));
            assert_ok!(BridgeModule::approve_transfer(
                Origin::signed(V2),
                sub_message_id
            ));

            message = get_message();
            assert_eq!(message.status, Status::Approved);

            // at this point transfer is in Approved status and are waiting for confirmation
            // from ethereum side to burn. Funds are locked.
            assert_eq!(TokenModule::locked((0, USER2)), token_amount);
            assert_eq!(TokenModule::balance_of((TOKEN_ID, USER2)), 0);
            // once it happends, validators call confirm_transfer

            assert_ok!(BridgeModule::confirm_transfer(
                Origin::signed(V2),
                sub_message_id
            ));

            message = get_message();
            let transfer = BridgeModule::transfers(1);
            assert_eq!(message.status, Status::Confirmed);
            assert_eq!(transfer.open, true);
            assert_ok!(BridgeModule::confirm_transfer(
                Origin::signed(V1),
                sub_message_id
            ));
            // assert_ok!(BridgeModule::confirm_transfer(Origin::signed(USER1), sub_message_id));
            //BurnedMessage(Hash, AccountId, H160, u64) event emitted
            let tokens_left = amount1 - token_amount;
            assert_eq!(TokenModule::balance_of((TOKEN_ID, USER2)), tokens_left);
            assert_eq!(TokenModule::total_supply(TOKEN_ID), tokens_left);

            assert_eq!(Balances::free_balance(DAO), 6000);
        })
    }
}
