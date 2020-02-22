/// runtime module implementing Substrate side of PolkadaiBridge token exchange bridge
/// You can use mint to create tokens backed by locked funds on Ethereum side
/// and transfer tokens on substrate side freely
///
/// KNOWN BUGS:
///     1. Tests can fail with assert_noop! bug: fails through different root hashes
///        solution: use assert_eq!(expr, Err("Error string")) explicitly
///
use crate::token;
use crate::types::{
    BridgeMessage, BridgeTransfer, Kind, LimitMessage, Limits, MemberId, ProposalId, Status,
    TokenBalance, TransferMessage, ValidatorMessage,
};
use codec::Encode;
use primitives::H160;
use rstd::prelude::Vec;
use sp_runtime::traits::Hash;
use support::{
    decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure, fail, StorageMap,
    StorageValue,
};
use system::{self, ensure_signed};

type Result<T> = core::result::Result<T, &'static str>;

const MAX_VALIDATORS: u32 = 100_000;
const DAY_IN_BLOCKS: u32 = 14_400;
const DAY: u32 = 86_400;

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Hash = <T as system::Trait>::Hash,
    {
        RelayMessage(Hash),
        ApprovedRelayMessage(Hash, AccountId, H160, TokenBalance),
        CancellationConfirmedMessage(Hash),
        MintedMessage(Hash),
        BurnedMessage(Hash, AccountId, H160, TokenBalance),
        AccountPausedMessage(Hash, AccountId),
        AccountResumedMessage(Hash, AccountId),
    }
);

pub trait Trait: token::Trait + system::Trait + timestamp::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Bridge {
        BridgeIsOperational get(bridge_is_operational): bool = true;
        BridgeMessages get(bridge_messages): map T::Hash  => BridgeMessage<T::AccountId, T::Hash>;

        // limits change history
        LimitMessages get(limit_messages): map T::Hash  => LimitMessage<T::Hash>;
        CurrentLimits get(current_limits) build(|config: &GenesisConfig<T>| {
            let mut limits_iter = config.current_limits.clone().into_iter();
            Limits {
                max_tx_value: limits_iter.next().unwrap(),
                day_max_limit: limits_iter.next().unwrap(),
                day_max_limit_for_one_address: limits_iter.next().unwrap(),
                max_pending_tx_limit: limits_iter.next().unwrap(),
                min_tx_value: limits_iter.next().unwrap(),
            }
        }): Limits;

        // open transactions
        CurrentPendingBurn get(pending_burn_count): u128;
        CurrentPendingMint get(pending_mint_count): u128;

        BridgeTransfers get(transfers): map ProposalId => BridgeTransfer<T::Hash>;
        BridgeTransfersCount get(bridge_transfers_count): ProposalId;
        TransferMessages get(messages): map T::Hash  => TransferMessage<T::AccountId, T::Hash>;
        TransferId get(transfer_id_by_hash): map T::Hash  => ProposalId;
        MessageId get(message_id_by_transfer_id): map ProposalId  => T::Hash;

        DailyHolds get(daily_holds): map T::AccountId  => (T::BlockNumber, T::Hash);
        DailyLimits get(daily_limits_by_account): map T::AccountId  => TokenBalance;
        DailyBlocked get(daily_blocked): map T::Moment  => Vec<T::AccountId>;

        Quorum get(quorum): u64 = 2;
        ValidatorsCount get(validators_count) config(): u32 = 3;
        ValidatorVotes get(validator_votes): map(ProposalId, T::AccountId) => bool;
        ValidatorHistory get(validator_history): map T::Hash  => ValidatorMessage<T::AccountId, T::Hash>;
        Validators get(validators) build(|config: &GenesisConfig<T>| {
            config.validator_accounts.clone().into_iter()
            .map(|acc: T::AccountId| (acc, true)).collect::<Vec<_>>()
        }): map T::AccountId  => bool;
        ValidatorAccounts get(validator_accounts) config(): Vec<T::AccountId>;
    }
    add_extra_genesis{
        config(current_limits): Vec<u128>;
}
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        // initiate substrate -> ethereum transfer.
        // create transfer and emit the RelayMessage event
        fn set_transfer(origin, to: H160, #[compact] amount: TokenBalance)-> DispatchResult
        {
            let from = ensure_signed(origin)?;
            ensure!(Self::bridge_is_operational(), "Bridge is not operational");

            let default_token = <token::Module<T>>::tokens(0).clone();
           <token::Module<T>>::check_token_exist(&default_token.symbol)?;
           let token_id = <token::Module<T>>::token_id_by_symbol(default_token.symbol);

            Self::check_amount(amount)?;
            Self::check_pending_burn(amount)?;
            Self::check_daily_account_volume(from.clone(), amount)?;

            let transfer_hash = (&from, &to, amount, <timestamp::Module<T>>::get()).using_encoded(<T as system::Trait>::Hashing::hash);

            let message = TransferMessage {
                message_id: transfer_hash,
                eth_address: to,
                substrate_address: from.clone(),
                amount,
                token: token_id,
                status: Status::Withdraw,
                action: Status::Withdraw,
            };
            Self::get_transfer_id_checked(transfer_hash, Kind::Transfer)?;
            Self::deposit_event(RawEvent::RelayMessage(transfer_hash));

            <DailyLimits<T>>::mutate(from, |a| *a += amount);
            <TransferMessages<T>>::insert(transfer_hash, message);
            Ok(())
        }

        // ethereum-side multi-signed mint operation
        fn multi_signed_mint(origin, message_id: T::Hash, from: H160, to: T::AccountId, #[compact] amount: TokenBalance)-> DispatchResult {
            let validator = ensure_signed(origin)?;
            ensure!(Self::bridge_is_operational(), "Bridge is not operational");

            let default_token = <token::Module<T>>::tokens(0).clone();
           <token::Module<T>>::check_token_exist(&default_token.symbol)?;
           let token_id = <token::Module<T>>::token_id_by_symbol(default_token.symbol);

            Self::check_validator(validator.clone())?;
            Self::check_pending_mint(amount)?;
            Self::check_amount(amount)?;

            if !<TransferMessages<T>>::exists(message_id) {
                let message = TransferMessage{
                    message_id,
                    eth_address: from,
                    substrate_address: to,
                    amount,
                    token: token_id,
                    status: Status::Deposit,
                    action: Status::Deposit,
                };
                <TransferMessages<T>>::insert(message_id, message);
                Self::get_transfer_id_checked(message_id, Kind::Transfer)?;
            }

            let transfer_id = <TransferId<T>>::get(message_id);
            Self::_sign(validator, transfer_id)?;
            Ok(())
        }

        // change maximum tx limit
        fn update_limits(origin, max_tx_value: u128, day_max_limit: u128, day_max_limit_for_one_address: u128, max_pending_tx_limit: u128,min_tx_value: u128)-> DispatchResult {
            let validator = ensure_signed(origin)?;
            Self::check_validator(validator.clone())?;
            let limits = Limits{
                max_tx_value,
                day_max_limit,
                day_max_limit_for_one_address,
                max_pending_tx_limit,
                min_tx_value,
            };
            Self::check_limits(&limits)?;
            let id = (limits.clone(), T::BlockNumber::from(0)).using_encoded(<T as system::Trait>::Hashing::hash);

            if !<LimitMessages<T>>::exists(id) {
                let message = LimitMessage {
                    id,
                    limits,
                    status: Status::UpdateLimits,
                };
                <LimitMessages<T>>::insert(id, message);
                Self::get_transfer_id_checked(id, Kind::Limits)?;
            }

            let transfer_id = <TransferId<T>>::get(id);
            Self::_sign(validator, transfer_id)?;
            Ok(())
        }

        // validator`s response to RelayMessage
        fn approve_transfer(origin, message_id: T::Hash) -> DispatchResult {
            let validator = ensure_signed(origin)?;
            ensure!(Self::bridge_is_operational(), "Bridge is not operational");
            Self::check_validator(validator.clone())?;

            let id = <TransferId<T>>::get(message_id);
            Self::_sign(validator, id)?;
            Ok(())
        }

        // each validator calls it to update whole set of validators
        fn update_validator_list(origin, message_id: T::Hash, quorum: u64, new_validator_list: Vec<T::AccountId>) -> DispatchResult {
            let validator = ensure_signed(origin)?;
            Self::check_validator(validator.clone())?;

            if !<ValidatorHistory<T>>::exists(message_id) {
                let message = ValidatorMessage {
                    message_id,
                    quorum,
                    accounts: new_validator_list,
                    action: Status::UpdateValidatorSet,
                    status: Status::UpdateValidatorSet,
                };
                <ValidatorHistory<T>>::insert(message_id, message);
                Self::get_transfer_id_checked(message_id, Kind::Validator)?;
            }

            let id = <TransferId<T>>::get(message_id);
            Self::_sign(validator, id)?;
            Ok(())
        }

        // each validator calls it to pause the bridge
        fn pause_bridge(origin) -> DispatchResult {
            let validator = ensure_signed(origin)?;
            Self::check_validator(validator.clone())?;

            ensure!(Self::bridge_is_operational(), "Bridge is not operational already");
            let hash = ("pause", T::BlockNumber::from(0)).using_encoded(<T as system::Trait>::Hashing::hash);

            if !<BridgeMessages<T>>::exists(hash) {
                let message = BridgeMessage {
                    message_id: hash,
                    account: validator.clone(),
                    action: Status::PauseTheBridge,
                    status: Status::PauseTheBridge,
                };
                <BridgeMessages<T>>::insert(hash, message);
                Self::get_transfer_id_checked(hash, Kind::Bridge)?;
            }

            let id = <TransferId<T>>::get(hash);
            Self::_sign(validator, id)?;
            Ok(())
        }

        // each validator calls it to resume the bridge
        fn resume_bridge(origin) -> DispatchResult {
            let validator = ensure_signed(origin)?;
            Self::check_validator(validator.clone())?;

            let hash = ("resume", T::BlockNumber::from(0)).using_encoded(<T as system::Trait>::Hashing::hash);

            if !<BridgeMessages<T>>::exists(hash) {
                let message = BridgeMessage {
                    message_id: hash,
                    account: validator.clone(),
                    action: Status::ResumeTheBridge,
                    status: Status::ResumeTheBridge,
                };
                <BridgeMessages<T>>::insert(hash, message);
                Self::get_transfer_id_checked(hash, Kind::Bridge)?;
            }

            let id = <TransferId<T>>::get(hash);
            Self::_sign(validator, id)?;
            Ok(())
        }

        //confirm burn from validator
        fn confirm_transfer(origin, message_id: T::Hash) -> DispatchResult {
            let validator = ensure_signed(origin)?;
            ensure!(Self::bridge_is_operational(), "Bridge is not operational");
            Self::check_validator(validator.clone())?;

            let id = <TransferId<T>>::get(message_id);

            let is_approved = <TransferMessages<T>>::get(message_id).status == Status::Approved ||
            <TransferMessages<T>>::get(message_id).status == Status::Confirmed;
            ensure!(is_approved, "This transfer must be approved first.");

            Self::update_status(message_id, Status::Confirmed, Kind::Transfer)?;
            Self::reopen_for_burn_confirmation(message_id)?;
            Self::_sign(validator, id)?;
            Ok(())
        }

        //cancel burn from validator
        fn cancel_transfer(origin, message_id: T::Hash) -> DispatchResult {
            let validator = ensure_signed(origin)?;
            Self::check_validator(validator.clone())?;

            let has_burned = <TransferMessages<T>>::exists(message_id) && <TransferMessages<T>>::get(message_id).status == Status::Confirmed;
            ensure!(!has_burned, "Failed to cancel. This transfer is already executed.");

            let id = <TransferId<T>>::get(message_id);
            Self::update_status(message_id, Status::Canceled, Kind::Transfer)?;
            Self::reopen_for_burn_confirmation(message_id)?;
            Self::_sign(validator, id)?;
            Ok(())
        }

        //close enough to clear it exactly at UTC 00:00 instead of BlockNumber
        fn on_finalize() {
            // clear accounts blocked day earlier (e.g. 18759 - 1)
            let yesterday = Self::get_day_pair().0;
            let is_first_day = Self::get_day_pair().1 == yesterday;
            if <DailyBlocked<T>>::exists(&yesterday) && !is_first_day {
                let blocked_yesterday = <DailyBlocked<T>>::get(&yesterday);
                blocked_yesterday.iter().for_each(|a| <DailyLimits<T>>::remove(a));
                blocked_yesterday.iter().for_each(|a|{
                    let hash = (<timestamp::Module<T>>::get(), a.clone()).using_encoded(<T as system::Trait>::Hashing::hash);
                    Self::deposit_event(RawEvent::AccountResumedMessage(hash, a.clone()));
                }
                );
                <DailyBlocked<T>>::remove(&yesterday);
            }
        }
    }
}

impl<T: Trait> Module<T> {
    fn _sign(validator: T::AccountId, transfer_id: ProposalId) -> Result<()> {
        let mut transfer = <BridgeTransfers<T>>::get(transfer_id);

        let mut message = <TransferMessages<T>>::get(transfer.message_id);
        let mut limit_message = <LimitMessages<T>>::get(transfer.message_id);
        let mut validator_message = <ValidatorHistory<T>>::get(transfer.message_id);
        let mut bridge_message = <BridgeMessages<T>>::get(transfer.message_id);
        let voted = <ValidatorVotes<T>>::get((transfer_id, validator.clone()));
        ensure!(!voted, "This validator has already voted.");
        ensure!(transfer.open, "This transfer is not open");
        transfer.votes += 1;

        if Self::votes_are_enough(transfer.votes) {
            match message.status {
                Status::Confirmed | Status::Canceled => (), // if burn is confirmed or canceled
                _ => match transfer.kind {
                    Kind::Transfer => message.status = Status::Approved,
                    Kind::Limits => limit_message.status = Status::Approved,
                    Kind::Validator => validator_message.status = Status::Approved,
                    Kind::Bridge => bridge_message.status = Status::Approved,
                },
            }
            match transfer.kind {
                Kind::Transfer => Self::execute_transfer(message)?,
                Kind::Limits => Self::_update_limits(limit_message)?,
                Kind::Validator => Self::manage_validator_list(validator_message)?,
                Kind::Bridge => Self::manage_bridge(bridge_message)?,
            }
            transfer.open = false;
        } else {
            match message.status {
                Status::Confirmed | Status::Canceled => (),
                _ => Self::set_pending(transfer_id, transfer.kind.clone())?,
            };
        }

        <ValidatorVotes<T>>::mutate((transfer_id, validator), |a| *a = true);
        <BridgeTransfers<T>>::insert(transfer_id, transfer);

        Ok(())
    }

    ///get (yesterday,today) pair
    fn get_day_pair() -> (T::Moment, T::Moment) {
        let now = <timestamp::Module<T>>::get();
        let day = T::Moment::from(DAY);
        let today = <timestamp::Module<T>>::get() / T::Moment::from(DAY);
        let yesterday = if now < day {
            T::Moment::from(0)
        } else {
            <timestamp::Module<T>>::get() / day - T::Moment::from(1)
        };
        (yesterday, today)
    }

    ///ensure that such transfer exist
    fn get_transfer_id_checked(transfer_hash: T::Hash, kind: Kind) -> Result<()> {
        if !<TransferId<T>>::exists(transfer_hash) {
            Self::create_transfer(transfer_hash, kind)?;
        }
        Ok(())
    }

    ///execute actual mint
    fn deposit(message: TransferMessage<T::AccountId, T::Hash>) -> Result<()> {
        Self::sub_pending_mint(message.clone())?;
        let to = message.substrate_address;
        if !<DailyHolds<T>>::exists(&to) {
            <DailyHolds<T>>::insert(to.clone(), (T::BlockNumber::from(0), message.message_id));
        }

        //TODO: implement actual token id instead of 0
        <token::Module<T>>::_mint(0, to, message.amount)?;

        Self::deposit_event(RawEvent::MintedMessage(message.message_id));
        Self::update_status(message.message_id, Status::Confirmed, Kind::Transfer)
    }

    fn withdraw(message: TransferMessage<T::AccountId, T::Hash>) -> Result<()> {
        Self::check_daily_holds(message.clone())?;
        Self::sub_pending_burn(message.clone())?;

        let to = message.eth_address;
        let from = message.substrate_address;
        Self::lock_for_burn(from.clone(), message.amount)?;
        Self::deposit_event(RawEvent::ApprovedRelayMessage(
            message.message_id,
            from,
            to,
            message.amount,
        ));
        Self::update_status(message.message_id, Status::Approved, Kind::Transfer)
    }
    fn _cancel_transfer(message: TransferMessage<T::AccountId, T::Hash>) -> Result<()> {
        //TODO: implement actual token id instead of 0
        <token::Module<T>>::unlock(0, &message.substrate_address, message.amount)?;
        Self::update_status(message.message_id, Status::Canceled, Kind::Transfer)
    }
    fn pause_the_bridge(message: BridgeMessage<T::AccountId, T::Hash>) -> Result<()> {
        <BridgeIsOperational>::mutate(|x| *x = false);
        Self::update_status(message.message_id, Status::Confirmed, Kind::Bridge)
    }

    fn resume_the_bridge(message: BridgeMessage<T::AccountId, T::Hash>) -> Result<()> {
        <BridgeIsOperational>::mutate(|x| *x = true);
        Self::update_status(message.message_id, Status::Confirmed, Kind::Bridge)
    }

    fn _update_limits(message: LimitMessage<T::Hash>) -> Result<()> {
        Self::check_limits(&message.limits)?;
        <CurrentLimits>::put(message.limits);
        Self::update_status(message.id, Status::Confirmed, Kind::Limits)
    }
    fn add_pending_burn(message: TransferMessage<T::AccountId, T::Hash>) -> Result<()> {
        let current = <CurrentPendingBurn>::get();
        let next = current
            .checked_add(message.amount)
            .ok_or("Overflow adding to new pending burn volume")?;
        <CurrentPendingBurn>::put(next);
        Ok(())
    }
    fn add_pending_mint(message: TransferMessage<T::AccountId, T::Hash>) -> Result<()> {
        let current = <CurrentPendingMint>::get();
        let next = current
            .checked_add(message.amount)
            .ok_or("Overflow adding to new pending mint volume")?;
        <CurrentPendingMint>::put(next);
        Ok(())
    }
    fn sub_pending_burn(message: TransferMessage<T::AccountId, T::Hash>) -> Result<()> {
        let current = <CurrentPendingBurn>::get();
        let next = current
            .checked_sub(message.amount)
            .ok_or("Overflow subtracting to new pending burn volume")?;
        <CurrentPendingBurn>::put(next);
        Ok(())
    }
    fn sub_pending_mint(message: TransferMessage<T::AccountId, T::Hash>) -> Result<()> {
        let current = <CurrentPendingMint>::get();
        let next = current
            .checked_sub(message.amount)
            .ok_or("Overflow subtracting to new pending mint volume")?;
        <CurrentPendingMint>::put(next);
        Ok(())
    }

    /// update validators list
    fn manage_validator_list(info: ValidatorMessage<T::AccountId, T::Hash>) -> Result<()> {
        let new_count = u32::from(info.accounts.clone().len());
        ensure!(
            new_count < MAX_VALIDATORS,
            "New validator list is exceeding allowed length."
        );
        <Quorum>::put(info.quorum);
        <ValidatorsCount>::put(new_count);
        info.accounts
            .clone()
            .iter()
            .for_each(|v| <Validators<T>>::insert(v, true));
        Self::update_status(info.message_id, Status::Confirmed, Kind::Validator)
    }

    /// check votes validity
    fn votes_are_enough(votes: MemberId) -> bool {
        votes as f64 / f64::from(Self::validators_count()) >= 0.51
    }

    /// lock funds after set_transfer call
    fn lock_for_burn(account: T::AccountId, amount: TokenBalance) -> Result<()> {
        //TODO: use token_id instead of 0
        <token::Module<T>>::lock(0, account, amount)?;

        Ok(())
    }

    fn execute_burn(message_id: T::Hash) -> Result<()> {
        let message = <TransferMessages<T>>::get(message_id);
        let from = message.substrate_address.clone();
        let to = message.eth_address;

        //TODO: implement actual token id instead of 0
        <token::Module<T>>::unlock(0, &from, message.amount)?;
        <token::Module<T>>::_burn(0, from.clone(), message.amount)?;
        <DailyLimits<T>>::mutate(from.clone(), |a| *a -= message.amount);

        Self::deposit_event(RawEvent::BurnedMessage(
            message_id,
            from,
            to,
            message.amount,
        ));
        Ok(())
    }

    fn execute_transfer(message: TransferMessage<T::AccountId, T::Hash>) -> Result<()> {
        match message.action {
            Status::Deposit => match message.status {
                Status::Approved => Self::deposit(message),
                Status::Canceled => Self::_cancel_transfer(message),
                _ => Err("Tried to deposit with non-supported status"),
            },
            Status::Withdraw => match message.status {
                Status::Confirmed => Self::execute_burn(message.message_id),
                Status::Approved => Self::withdraw(message),
                Status::Canceled => Self::_cancel_transfer(message),
                _ => Err("Tried to withdraw with non-supported status"),
            },
            _ => Err("Tried to execute transfer with non-supported status"),
        }
    }

    fn manage_bridge(message: BridgeMessage<T::AccountId, T::Hash>) -> Result<()> {
        match message.action {
            Status::PauseTheBridge => match message.status {
                Status::Approved => Self::pause_the_bridge(message),
                _ => Err("Tried to pause the bridge with non-supported status"),
            },
            Status::ResumeTheBridge => match message.status {
                Status::Approved => Self::resume_the_bridge(message),
                _ => Err("Tried to resume the bridge with non-supported status"),
            },
            _ => Err("Tried to manage bridge with non-supported status"),
        }
    }

    fn create_transfer(transfer_hash: T::Hash, kind: Kind) -> Result<()> {
        ensure!(
            !<TransferId<T>>::exists(transfer_hash),
            "This transfer already open"
        );

        let transfer_id = <BridgeTransfersCount>::get();
        let bridge_transfers_count = <BridgeTransfersCount>::get();
        let new_bridge_transfers_count = bridge_transfers_count
            .checked_add(1)
            .ok_or("Overflow adding a new bridge transfer")?;
        let transfer = BridgeTransfer {
            transfer_id,
            message_id: transfer_hash,
            open: true,
            votes: 0,
            kind,
        };

        <BridgeTransfers<T>>::insert(transfer_id, transfer);
        <BridgeTransfersCount>::mutate(|count| *count = new_bridge_transfers_count);
        <TransferId<T>>::insert(transfer_hash, transfer_id);
        <MessageId<T>>::insert(transfer_id, transfer_hash);

        Ok(())
    }

    fn set_pending(transfer_id: ProposalId, kind: Kind) -> Result<()> {
        let message_id = <MessageId<T>>::get(transfer_id);
        match kind {
            Kind::Transfer => {
                let message = <TransferMessages<T>>::get(message_id);
                match message.action {
                    Status::Withdraw => Self::add_pending_burn(message)?,
                    Status::Deposit => Self::add_pending_mint(message)?,
                    _ => (),
                }
            }
            _ => (),
        }
        Self::update_status(message_id, Status::Pending, kind)
    }

    fn update_status(id: T::Hash, status: Status, kind: Kind) -> Result<()> {
        match kind {
            Kind::Transfer => {
                let mut message = <TransferMessages<T>>::get(id);
                message.status = status;
                <TransferMessages<T>>::insert(id, message);
            }
            Kind::Validator => {
                let mut message = <ValidatorHistory<T>>::get(id);
                message.status = status;
                <ValidatorHistory<T>>::insert(id, message);
            }
            Kind::Bridge => {
                let mut message = <BridgeMessages<T>>::get(id);
                message.status = status;
                <BridgeMessages<T>>::insert(id, message);
            }
            Kind::Limits => {
                let mut message = <LimitMessages<T>>::get(id);
                message.status = status;
                <LimitMessages<T>>::insert(id, message);
            }
        }
        Ok(())
    }

    // needed because @message_id will be the same as initial
    fn reopen_for_burn_confirmation(message_id: T::Hash) -> Result<()> {
        let message = <TransferMessages<T>>::get(message_id);
        let transfer_id = <TransferId<T>>::get(message_id);
        let mut transfer = <BridgeTransfers<T>>::get(transfer_id);
        let is_eth_response =
            message.status == Status::Confirmed || message.status == Status::Canceled;
        if !transfer.open && is_eth_response {
            transfer.votes = 0;
            transfer.open = true;
            <BridgeTransfers<T>>::insert(transfer_id, transfer);
            let validators = <ValidatorAccounts<T>>::get();
            validators
                .iter()
                .for_each(|a| <ValidatorVotes<T>>::insert((transfer_id, a.clone()), false));
        }
        Ok(())
    }
    fn check_validator(validator: T::AccountId) -> Result<()> {
        let is_trusted = <Validators<T>>::exists(validator);
        ensure!(is_trusted, "Only validators can call this function");

        Ok(())
    }

    fn check_daily_account_volume(account: T::AccountId, amount: TokenBalance) -> Result<()> {
        let cur_pending = <DailyLimits<T>>::get(&account);
        let cur_pending_account_limit = <CurrentLimits>::get().day_max_limit_for_one_address;
        let can_burn = cur_pending + amount < cur_pending_account_limit;

        //store current day (like 18768)
        let today = Self::get_day_pair().1;
        let user_blocked = <DailyBlocked<T>>::get(&today).iter().any(|a| *a == account);

        if !can_burn {
            <DailyBlocked<T>>::mutate(today, |v| {
                if !v.contains(&account) {
                    v.push(account.clone());
                    let hash = (<timestamp::Module<T>>::get(), account.clone())
                        .using_encoded(<T as system::Trait>::Hashing::hash);
                    Self::deposit_event(RawEvent::AccountPausedMessage(hash, account))
                }
            });
        }
        ensure!(
            can_burn && !user_blocked,
            "Transfer declined, user blocked due to daily volume limit."
        );

        Ok(())
    }
    fn check_amount(amount: TokenBalance) -> Result<()> {
        let max = <CurrentLimits>::get().max_tx_value;
        let min = <CurrentLimits>::get().min_tx_value;

        ensure!(
            amount > min,
            "Invalid amount for transaction. Reached minimum limit."
        );
        ensure!(
            amount < max,
            "Invalid amount for transaction. Reached maximum limit."
        );
        Ok(())
    }
    //open transactions check
    fn check_pending_burn(amount: TokenBalance) -> Result<()> {
        let new_pending_volume = <CurrentPendingBurn>::get()
            .checked_add(amount)
            .ok_or("Overflow adding to new pending burn volume")?;
        let can_burn = new_pending_volume < <CurrentLimits>::get().max_pending_tx_limit;
        ensure!(can_burn, "Too many pending burn transactions.");
        Ok(())
    }

    fn check_pending_mint(amount: TokenBalance) -> Result<()> {
        let new_pending_volume = <CurrentPendingMint>::get()
            .checked_add(amount)
            .ok_or("Overflow adding to new pending mint volume")?;
        let can_burn = new_pending_volume < <CurrentLimits>::get().max_pending_tx_limit;
        ensure!(can_burn, "Too many pending mint transactions.");
        Ok(())
    }

    fn check_limits(limits: &Limits) -> Result<()> {
        let max = u128::max_value();
        let min = u128::min_value();
        let passed = limits
            .into_array()
            .iter()
            .fold((true, true), |acc, l| match acc {
                (true, true) => (l < &max, l > &min),
                (true, false) => (l < &max, false),
                (false, true) => (false, l > &min),
                (_, _) => acc,
            });
        ensure!(passed.0, "Overflow setting limit");
        ensure!(passed.1, "Underflow setting limit");
        Ok(())
    }

    fn check_daily_holds(message: TransferMessage<T::AccountId, T::Hash>) -> Result<()> {
        let from = message.substrate_address;
        let first_tx = <DailyHolds<T>>::get(from.clone());
        let daily_hold = T::BlockNumber::from(DAY_IN_BLOCKS);
        let day_passed = first_tx.0 + daily_hold < T::BlockNumber::from(0);

        if !day_passed {
            let account_balance = <token::Module<T>>::balance_of((0, from));
            // 75% of potentially really big numbers
            let allowed_amount = account_balance
                .checked_div(100)
                .expect("Failed to calculate allowed withdraw amount")
                .checked_mul(75)
                .expect("Failed to calculate allowed withdraw amount");

            if message.amount > allowed_amount {
                Self::update_status(message.message_id, Status::Canceled, Kind::Transfer)?;
                fail!("Cannot withdraw more that 75% of first day deposit.");
            }
        }

        Ok(())
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;
    //TODO: fix limits after adding them into config
    use crate::types::Token;
    use primitives::{Blake2Hasher, H160, H256};
    use runtime_io::with_externalities;
    use sp_runtime::{
        testing::{Digest, DigestItem, Header},
        traits::{BlakeTwo256, IdentityLookup, OnFinalize},
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
    impl token::Trait for Test {
        type Event = ();
    }
    impl Trait for Test {
        type Event = ();
    }

    type BridgeModule = Module<Test>;
    type TokenModule = token::Module<Test>;
    type TimestampModule = timestamp::Module<Test>;
    type System = system::Module<Test>;

    const ETH_MESSAGE_ID: &[u8; 32] = b"0x5617efe391571b5dc8230db92ba65b";
    const ETH_MESSAGE_ID1: &[u8; 32] = b"0x5617iru391571b5dc8230db92ba65b";
    const ETH_MESSAGE_ID2: &[u8; 32] = b"0x5617yhk391571b5dc8230db92ba65b";
    const ETH_MESSAGE_ID3: &[u8; 32] = b"0x5617jdp391571b5dc8230db92ba65b";
    const ETH_MESSAGE_ID4: &[u8; 32] = b"0x5617kpt391571b5dc8230db92ba65b";
    const ETH_MESSAGE_ID5: &[u8; 32] = b"0x5617oet391571b5dc8230db92ba65b";
    const ETH_MESSAGE_ID6: &[u8; 32] = b"0x5617pey391571b5dc8230db92ba65b";
    const ETH_MESSAGE_ID7: &[u8; 32] = b"0x5617jqu391571b5dc8230db92ba65b";
    const ETH_MESSAGE_ID8: &[u8; 32] = b"0x5617pbt391571b5dc8230db92ba65b";
    const ETH_ADDRESS: &[u8; 20] = b"0x00b46c2526ebb8f4c9";
    const V1: u64 = 1;
    const V2: u64 = 2;
    const V3: u64 = 3;
    const V4: u64 = 4;
    const USER1: u64 = 5;
    const USER2: u64 = 6;
    const USER3: u64 = 7;
    const USER4: u64 = 8;
    const USER5: u64 = 9;
    const USER6: u64 = 10;
    const USER7: u64 = 11;
    const USER8: u64 = 12;
    const USER9: u64 = 13;

    //fast forward approximately
    fn run_to_block(n: u64) {
        while System::block_number() < n {
            BridgeModule::on_finalize(System::block_number());
            TimestampModule::set_timestamp(6 * n);
            System::set_block_number(System::block_number() + 1);
        }
    }

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        let mut r = system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap()
            .0;

        //balances chain_spec configuration
        r.extend(
            balances::GenesisConfig::<Test> {
                balances: vec![
                    (V1, 100000),
                    (V2, 100000),
                    (V3, 100000),
                    (USER1, 100000),
                    (USER2, 300000),
                ],
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
        //token chain_spec configuration
        r.extend(
            token::GenesisConfig::<Test> {
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
        //bridge chain_spec configuration
        r.extend(
            GenesisConfig::<Test> {
                validators_count: 3u32,
                validator_accounts: vec![V1, V2, V3],
                current_limits: vec![100, 200, 50, 400, 1],
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
            let message_id = H256::from(ETH_MESSAGE_ID);
            let eth_address = H160::from(ETH_ADDRESS);
            let amount = 99;

            let token = TokenModule::tokens(0);
            println!("{:?}", token);

            //substrate <----- ETH
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V2),
                message_id,
                eth_address,
                USER2,
                amount
            ));
            let mut message = BridgeModule::messages(message_id);
            assert_eq!(message.status, Status::Pending);

            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V1),
                message_id,
                eth_address,
                USER2,
                amount
            ));
            message = BridgeModule::messages(message_id);
            assert_eq!(message.status, Status::Confirmed);

            let transfer = BridgeModule::transfers(0);
            assert_eq!(transfer.open, false);

            assert_eq!(TokenModule::balance_of((0, USER2)), amount);
            assert_eq!(TokenModule::total_supply(0), amount);
        })
    }
    #[test]
    fn token_eth2sub_closed_transfer_fail() {
        with_externalities(&mut new_test_ext(), || {
            let message_id = H256::from(ETH_MESSAGE_ID);
            let eth_address = H160::from(ETH_ADDRESS);
            let amount = 99;

            //substrate <----- ETH
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V2),
                message_id,
                eth_address,
                USER2,
                amount
            ));
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V1),
                message_id,
                eth_address,
                USER2,
                amount
            ));
            assert_noop!(
                BridgeModule::multi_signed_mint(
                    Origin::signed(V3),
                    message_id,
                    eth_address,
                    USER2,
                    amount
                ),
                "This transfer is not open"
            );
            assert_eq!(TokenModule::balance_of((0, USER2)), amount);
            assert_eq!(TokenModule::total_supply(0), amount);
            let transfer = BridgeModule::transfers(0);
            assert_eq!(transfer.open, false);

            let message = BridgeModule::messages(message_id);
            assert_eq!(message.status, Status::Confirmed);
        })
    }

    #[test]
    fn token_sub2eth_burn_works() {
        with_externalities(&mut new_test_ext(), || {
            let eth_address = H160::from(ETH_ADDRESS);
            let amount1 = 600;
            let amount2 = 49;

            let _ = TokenModule::_mint(0, USER2, amount1);

            //substrate ----> ETH
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER2),
                eth_address,
                amount2
            ));
            //RelayMessage(message_id) event emitted

            let sub_message_id = BridgeModule::message_id_by_transfer_id(0);
            let get_message = || BridgeModule::messages(sub_message_id);

            let mut message = get_message();
            assert_eq!(message.status, Status::Withdraw);

            //approval
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
            assert_eq!(TokenModule::locked((0, USER2)), amount2);
            assert_eq!(TokenModule::balance_of((0, USER2)), amount1);
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
            let tokens_left = amount1 - amount2;
            assert_eq!(TokenModule::balance_of((0, USER2)), tokens_left);
            assert_eq!(TokenModule::total_supply(0), tokens_left);
        })
    }
    #[test]
    fn token_sub2eth_burn_skipped_approval_should_fail() {
        with_externalities(&mut new_test_ext(), || {
            let eth_address = H160::from(ETH_ADDRESS);
            let amount1 = 600;
            let amount2 = 49;

            let _ = TokenModule::_mint(0, USER2, amount1);

            assert_eq!(TokenModule::balance_of((0, USER2)), amount1);
            assert_eq!(TokenModule::total_supply(0), amount1);

            //substrate ----> ETH
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER2),
                eth_address,
                amount2
            ));
            //RelayMessage(message_id) event emitted

            let sub_message_id = BridgeModule::message_id_by_transfer_id(0);
            let message = BridgeModule::messages(sub_message_id);
            assert_eq!(message.status, Status::Withdraw);

            assert_eq!(TokenModule::locked((0, USER2)), 0);
            // lets say validators blacked out and we
            // try to confirm without approval anyway
            assert_noop!(
                BridgeModule::confirm_transfer(Origin::signed(V1), sub_message_id),
                "This transfer must be approved first."
            );
        })
    }
    #[test]
    fn token_sub2eth_burn_cancel_works() {
        with_externalities(&mut new_test_ext(), || {
            let eth_address = H160::from(ETH_ADDRESS);
            let amount1 = 600;
            let amount2 = 49;

            let _ = TokenModule::_mint(0, USER2, amount1);

            //substrate ----> ETH
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER2),
                eth_address,
                amount2
            ));

            let sub_message_id = BridgeModule::message_id_by_transfer_id(0);
            assert_ok!(BridgeModule::approve_transfer(
                Origin::signed(V1),
                sub_message_id
            ));
            assert_ok!(BridgeModule::approve_transfer(
                Origin::signed(V2),
                sub_message_id
            ));
            let mut message = BridgeModule::messages(sub_message_id);
            // funds are locked and waiting for confirmation
            assert_eq!(message.status, Status::Approved);
            assert_ok!(BridgeModule::cancel_transfer(
                Origin::signed(V2),
                sub_message_id
            ));
            assert_ok!(BridgeModule::cancel_transfer(
                Origin::signed(V3),
                sub_message_id
            ));
            message = BridgeModule::messages(sub_message_id);
            assert_eq!(message.status, Status::Canceled);
        })
    }
    #[test]
    fn burn_cancel_should_fail() {
        with_externalities(&mut new_test_ext(), || {
            let eth_address = H160::from(ETH_ADDRESS);
            let amount1 = 600;
            let amount2 = 49;

            let _ = TokenModule::_mint(0, USER2, amount1);

            //substrate ----> ETH
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER2),
                eth_address,
                amount2
            ));

            let sub_message_id = BridgeModule::message_id_by_transfer_id(0);
            let get_message = || BridgeModule::messages(sub_message_id);

            let mut message = get_message();
            assert_eq!(message.status, Status::Withdraw);

            //approval
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
            assert_eq!(TokenModule::locked((0, USER2)), amount2);
            assert_eq!(TokenModule::balance_of((0, USER2)), amount1);
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
            let tokens_left = amount1 - amount2;
            assert_eq!(TokenModule::balance_of((0, USER2)), tokens_left);
            assert_eq!(TokenModule::total_supply(0), tokens_left);
            assert_noop!(
                BridgeModule::cancel_transfer(Origin::signed(V2), sub_message_id),
                "Failed to cancel. This transfer is already executed."
            );
        })
    }
    #[test]
    fn update_validator_list_should_work() {
        with_externalities(&mut new_test_ext(), || {
            let eth_message_id = H256::from(ETH_MESSAGE_ID);
            const QUORUM: u64 = 3;

            assert_ok!(BridgeModule::update_validator_list(
                Origin::signed(V2),
                eth_message_id,
                QUORUM,
                vec![V1, V2, V3, V4]
            ));
            let id = BridgeModule::message_id_by_transfer_id(0);
            let mut message = BridgeModule::validator_history(id);
            assert_eq!(message.status, Status::Pending);

            assert_ok!(BridgeModule::update_validator_list(
                Origin::signed(V1),
                eth_message_id,
                QUORUM,
                vec![V1, V2, V3, V4]
            ));
            message = BridgeModule::validator_history(id);
            assert_eq!(message.status, Status::Confirmed);
            assert_eq!(BridgeModule::validators_count(), 4);
        })
    }
    #[test]
    fn pause_the_bridge_should_work() {
        with_externalities(&mut new_test_ext(), || {
            assert_ok!(BridgeModule::pause_bridge(Origin::signed(V2)));

            assert_eq!(BridgeModule::bridge_transfers_count(), 1);
            assert_eq!(BridgeModule::bridge_is_operational(), true);
            let id = BridgeModule::message_id_by_transfer_id(0);
            let mut message = BridgeModule::bridge_messages(id);
            assert_eq!(message.status, Status::Pending);

            assert_ok!(BridgeModule::pause_bridge(Origin::signed(V1)));
            assert_eq!(BridgeModule::bridge_is_operational(), false);
            message = BridgeModule::bridge_messages(id);
            assert_eq!(message.status, Status::Confirmed);
        })
    }
    #[test]
    fn extrinsics_restricted_should_fail() {
        with_externalities(&mut new_test_ext(), || {
            let eth_message_id = H256::from(ETH_MESSAGE_ID);
            let eth_address = H160::from(ETH_ADDRESS);

            assert_ok!(BridgeModule::pause_bridge(Origin::signed(V2)));
            assert_ok!(BridgeModule::pause_bridge(Origin::signed(V1)));

            // substrate <-- Ethereum
            assert_noop!(
                BridgeModule::multi_signed_mint(
                    Origin::signed(V2),
                    eth_message_id,
                    eth_address,
                    USER2,
                    1000
                ),
                "Bridge is not operational"
            );
        })
    }
    #[test]
    fn double_pause_should_fail() {
        with_externalities(&mut new_test_ext(), || {
            assert_eq!(BridgeModule::bridge_is_operational(), true);
            assert_ok!(BridgeModule::pause_bridge(Origin::signed(V2)));
            assert_ok!(BridgeModule::pause_bridge(Origin::signed(V1)));
            assert_eq!(BridgeModule::bridge_is_operational(), false);
            assert_noop!(
                BridgeModule::pause_bridge(Origin::signed(V1)),
                "Bridge is not operational already"
            );
        })
    }
    #[test]
    fn pause_and_resume_the_bridge_should_work() {
        with_externalities(&mut new_test_ext(), || {
            assert_eq!(BridgeModule::bridge_is_operational(), true);
            assert_ok!(BridgeModule::pause_bridge(Origin::signed(V2)));
            assert_ok!(BridgeModule::pause_bridge(Origin::signed(V1)));
            assert_eq!(BridgeModule::bridge_is_operational(), false);
            assert_ok!(BridgeModule::resume_bridge(Origin::signed(V1)));
            assert_ok!(BridgeModule::resume_bridge(Origin::signed(V2)));
            assert_eq!(BridgeModule::bridge_is_operational(), true);
        })
    }
    #[test]
    fn double_vote_should_fail() {
        with_externalities(&mut new_test_ext(), || {
            assert_eq!(BridgeModule::bridge_is_operational(), true);
            assert_ok!(BridgeModule::pause_bridge(Origin::signed(V2)));
            assert_noop!(
                BridgeModule::pause_bridge(Origin::signed(V2)),
                "This validator has already voted."
            );
        })
    }
    #[test]
    fn instant_withdraw_should_fail() {
        with_externalities(&mut new_test_ext(), || {
            let eth_message_id = H256::from(ETH_MESSAGE_ID);
            let eth_address = H160::from(ETH_ADDRESS);
            let amount1 = 99;
            let amount2 = 49;

            //substrate <----- ETH
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V2),
                eth_message_id,
                eth_address,
                USER2,
                amount1
            ));
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V1),
                eth_message_id,
                eth_address,
                USER2,
                amount1
            ));
            //substrate ----> ETH
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER2),
                eth_address,
                amount2
            ));
            //RelayMessage(message_id) event emitted
            let sub_message_id = BridgeModule::message_id_by_transfer_id(1);
            let get_message = || BridgeModule::messages(sub_message_id);
            let mut message = get_message();
            assert_eq!(message.status, Status::Withdraw);
            //approval
            assert_eq!(TokenModule::locked((0, USER2)), 0);
            assert_ok!(BridgeModule::approve_transfer(
                Origin::signed(V1),
                sub_message_id
            ));
            // assert_noop BUG: fails through different root hashes
            // solution: use assert_eq!(expr, Err("Error string")) explicitly
            assert_eq!(
                BridgeModule::approve_transfer(Origin::signed(V2), sub_message_id),
                Err("Cannot withdraw more that 75% of first day deposit.")
            );

            message = get_message();
            assert_eq!(message.status, Status::Canceled);
        })
    }
    #[test]
    fn change_limits_should_work() {
        with_externalities(&mut new_test_ext(), || {
            let max_tx_value = 10;
            let day_max_limit = 20;
            let day_max_limit_for_one_address = 5;
            let max_pending_tx_limit = 40;
            let min_tx_value = 1;

            assert_eq!(BridgeModule::current_limits().max_tx_value, 100);
            assert_ok!(BridgeModule::update_limits(
                Origin::signed(V2),
                max_tx_value,
                day_max_limit,
                day_max_limit_for_one_address,
                max_pending_tx_limit,
                min_tx_value,
            ));
            assert_ok!(BridgeModule::update_limits(
                Origin::signed(V1),
                max_tx_value,
                day_max_limit,
                day_max_limit_for_one_address,
                max_pending_tx_limit,
                min_tx_value,
            ));

            assert_eq!(BridgeModule::current_limits().max_tx_value, 10);
        })
    }
    #[test]
    fn change_limits_should_fail() {
        with_externalities(&mut new_test_ext(), || {
            let day_max_limit = 20;
            let day_max_limit_for_one_address = 5;
            let max_pending_tx_limit = 40;
            let min_tx_value = 1;
            const MORE_THAN_MAX: u128 = u128::max_value();

            assert_noop!(
                BridgeModule::update_limits(
                    Origin::signed(V1),
                    MORE_THAN_MAX,
                    day_max_limit,
                    day_max_limit_for_one_address,
                    max_pending_tx_limit,
                    min_tx_value,
                ),
                "Overflow setting limit"
            );
        })
    }
    #[test]
    fn pending_burn_limit_should_work() {
        with_externalities(&mut new_test_ext(), || {
            let eth_address = H160::from(ETH_ADDRESS);
            let amount1 = 60;
            let amount2 = 49;
            //TODO: pending transactions volume never reached if daily limit is lower
            let _ = TokenModule::_mint(0, USER1, amount1);
            let _ = TokenModule::_mint(0, USER2, amount1);
            let _ = TokenModule::_mint(0, USER3, amount1);
            let _ = TokenModule::_mint(0, USER4, amount1);
            let _ = TokenModule::_mint(0, USER5, amount1);
            let _ = TokenModule::_mint(0, USER6, amount1);
            let _ = TokenModule::_mint(0, USER7, amount1);
            let _ = TokenModule::_mint(0, USER8, amount1);
            let _ = TokenModule::_mint(0, USER9, amount1);
            //1
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER2),
                eth_address,
                amount2
            ));
            let sub_message_id = BridgeModule::message_id_by_transfer_id(0);
            assert_ok!(BridgeModule::approve_transfer(
                Origin::signed(V1),
                sub_message_id
            ));
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER3),
                eth_address,
                amount2
            ));
            let sub_message_id = BridgeModule::message_id_by_transfer_id(1);
            assert_ok!(BridgeModule::approve_transfer(
                Origin::signed(V1),
                sub_message_id
            ));
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER4),
                eth_address,
                amount2
            ));
            let sub_message_id = BridgeModule::message_id_by_transfer_id(2);
            assert_ok!(BridgeModule::approve_transfer(
                Origin::signed(V1),
                sub_message_id
            ));
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER5),
                eth_address,
                amount2
            ));
            let sub_message_id = BridgeModule::message_id_by_transfer_id(3);
            assert_ok!(BridgeModule::approve_transfer(
                Origin::signed(V1),
                sub_message_id
            ));
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER6),
                eth_address,
                amount2
            ));
            let sub_message_id = BridgeModule::message_id_by_transfer_id(4);
            assert_ok!(BridgeModule::approve_transfer(
                Origin::signed(V1),
                sub_message_id
            ));
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER7),
                eth_address,
                amount2
            ));
            let sub_message_id = BridgeModule::message_id_by_transfer_id(5);
            assert_ok!(BridgeModule::approve_transfer(
                Origin::signed(V1),
                sub_message_id
            ));
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER8),
                eth_address,
                amount2
            ));
            let sub_message_id = BridgeModule::message_id_by_transfer_id(6);
            assert_ok!(BridgeModule::approve_transfer(
                Origin::signed(V1),
                sub_message_id
            ));
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER9),
                eth_address,
                amount2
            ));
            let sub_message_id = BridgeModule::message_id_by_transfer_id(7);
            assert_ok!(BridgeModule::approve_transfer(
                Origin::signed(V1),
                sub_message_id
            ));

            assert_eq!(BridgeModule::pending_burn_count(), amount2 * 8);
            assert_noop!(
                BridgeModule::set_transfer(Origin::signed(USER1), eth_address, amount2),
                "Too many pending burn transactions."
            );
        })
    }
    #[test]
    fn pending_mint_limit_should_work() {
        with_externalities(&mut new_test_ext(), || {
            let eth_message_id = H256::from(ETH_MESSAGE_ID);
            let eth_message_id1 = H256::from(ETH_MESSAGE_ID1);
            let eth_message_id2 = H256::from(ETH_MESSAGE_ID2);
            let eth_message_id3 = H256::from(ETH_MESSAGE_ID3);
            let eth_message_id4 = H256::from(ETH_MESSAGE_ID4);
            let eth_message_id5 = H256::from(ETH_MESSAGE_ID5);
            let eth_message_id6 = H256::from(ETH_MESSAGE_ID6);
            let eth_message_id7 = H256::from(ETH_MESSAGE_ID7);
            let eth_message_id8 = H256::from(ETH_MESSAGE_ID8);
            let eth_address = H160::from(ETH_ADDRESS);
            let amount1 = 49;

            //substrate <----- ETH
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V2),
                eth_message_id,
                eth_address,
                USER2,
                amount1
            ));

            //substrate <----- ETH
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V2),
                eth_message_id2,
                eth_address,
                USER3,
                amount1
            ));

            //substrate <----- ETH
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V2),
                eth_message_id3,
                eth_address,
                USER4,
                amount1
            ));

            //substrate <----- ETH
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V2),
                eth_message_id4,
                eth_address,
                USER5,
                amount1
            ));
            //substrate <----- ETH
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V2),
                eth_message_id5,
                eth_address,
                USER6,
                amount1
            ));
            //substrate <----- ETH
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V2),
                eth_message_id6,
                eth_address,
                USER7,
                amount1
            ));
            //substrate <----- ETH
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V2),
                eth_message_id7,
                eth_address,
                USER8,
                amount1
            ));
            //substrate <----- ETH
            assert_ok!(BridgeModule::multi_signed_mint(
                Origin::signed(V2),
                eth_message_id8,
                eth_address,
                USER9,
                amount1
            ));
            assert_eq!(BridgeModule::pending_mint_count(), amount1 * 8);

            //substrate <----- ETH
            assert_noop!(
                BridgeModule::multi_signed_mint(
                    Origin::signed(V2),
                    eth_message_id1,
                    eth_address,
                    USER1,
                    amount1 + 5
                ),
                "Too many pending mint transactions."
            );
        })
    }
    #[test]
    fn blocking_account_by_volume_should_work() {
        with_externalities(&mut new_test_ext(), || {
            let eth_address = H160::from(ETH_ADDRESS);
            let amount1 = 600;
            let amount2 = 49;
            let _ = TokenModule::_mint(0, USER2, amount1);
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER2),
                eth_address,
                amount2
            ));
            let sub_message_id = BridgeModule::message_id_by_transfer_id(0);
            assert_ok!(BridgeModule::approve_transfer(
                Origin::signed(V1),
                sub_message_id
            ));
            assert_ok!(BridgeModule::approve_transfer(
                Origin::signed(V2),
                sub_message_id
            ));

            assert_eq!(
                BridgeModule::set_transfer(Origin::signed(USER2), eth_address, amount2),
                Err("Transfer declined, user blocked due to daily volume limit.")
            );
        })
    }
    #[test]
    fn blocked_account_unblocked_next_day_should_work() {
        with_externalities(&mut new_test_ext(), || {
            let eth_address = H160::from(ETH_ADDRESS);
            let amount1 = 600;
            let amount2 = 49;
            run_to_block(DAY_IN_BLOCKS);

            let _ = TokenModule::_mint(0, USER2, amount1);
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER2),
                eth_address,
                amount2
            ));
            let sub_message_id = BridgeModule::message_id_by_transfer_id(0);
            assert_ok!(BridgeModule::approve_transfer(
                Origin::signed(V1),
                sub_message_id
            ));
            assert_ok!(BridgeModule::approve_transfer(
                Origin::signed(V2),
                sub_message_id
            ));
            assert_eq!(
                BridgeModule::set_transfer(Origin::signed(USER2), eth_address, amount2),
                Err("Transfer declined, user blocked due to daily volume limit.")
            );

            //user added to blocked vec
            let blocked_vec: Vec<u64> = vec![USER2];
            assert_eq!(BridgeModule::daily_blocked(1), blocked_vec);

            run_to_block(DAY_IN_BLOCKS * 2);
            run_to_block(DAY_IN_BLOCKS * 3);

            //try again
            assert_ok!(BridgeModule::set_transfer(
                Origin::signed(USER2),
                eth_address,
                amount2
            ));
        })
    }
}
