// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use dispatch::DispatchResult;
use frame_support::traits::{OnUnbalanced, Vec};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure,
    traits::{Currency, ExistenceRequirement, LockIdentifier, LockableCurrency, WithdrawReasons},
    StorageMap,
};

use primitives::BlockNumber;
use sp_runtime::{
    traits::{Hash, Zero},
    Perbill, RuntimeDebug,
};

use sp_std::collections::btree_map::BTreeMap;
use sp_std::prelude::*;

mod offline;
pub use offline::GeodeOffence;
pub use offline::{
    AutomataOffence, AutomataOffenceDetails, AutomataOffenceError, AutomataReportOffence, GeodeIdd,
    Kind, OffenceReason, OffenceTempRecord, OnAutomataOffenceHandler, SlashParms,
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type Weight = u64;
const MARKETPLACE_ID: LockIdentifier = *b"market  ";

/// users' ledger for automata staking
#[derive(Default, PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct UserLedger<AccountId, Balance>
where
    AccountId: Ord,
{
    pub who: AccountId,
    pub total: Balance,
    pub active: Balance,
    pub buyer: BTreeMap<AccountId, Balance>,
    pub seller: BTreeMap<AccountId, Balance>,
}

pub type UserLedgerOf<T> = UserLedger<<T as frame_system::Config>::AccountId, BalanceOf<T>>;

pub type OrderId<T> = <T as frame_system::Config>::AccountId;
pub type DealId<T> = <T as frame_system::Config>::AccountId;
pub type GeodeId<T> = <T as frame_system::Config>::AccountId;
type ReportIdOf<T> = <T as frame_system::Config>::Hash;
pub type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type OffenceTempRecordOf<T> = OffenceTempRecord<<T as frame_system::Config>::AccountId>;

type PositiveImbalanceOf<T> = <<T as Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::PositiveImbalance;
type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

type CurrentOffline = u32;
type TotalOffline = u32;

pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

    type Currency: Currency<Self::AccountId> + LockableCurrency<Self::AccountId>;

    type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;

    type Reward: OnUnbalanced<PositiveImbalanceOf<Self>>;
}

decl_storage! {
    trait Store for Module<T: Config> as Stake {
            /// Users' Ledger
            Users get(fn users): map hasher(twox_64_concat) T::AccountId => UserLedgerOf<T>;

            /// record of all offence reports
            Reports get(fn reports): map hasher(twox_64_concat)
            ReportIdOf<T> => Option<AutomataOffenceDetails<T::AccountId, T::BlockNumber>>;

            /// Slash & Reward Rules
            SlashRule get(fn slash_rule): SlashParms;
            RewardRule get(fn reward_rule): Perbill;

            /// record of temp offence reports
            TempOffence get(fn temp_offence): map hasher(twox_64_concat) T::AccountId => Option<OffenceTempRecordOf<T>>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        Balance = BalanceOf<T>,
    {
        /// when a user is slashed.(AccountId, GeodeId, OrderId, DealId, Balance)
        Slashed(AccountId, AccountId, AccountId, AccountId, Balance),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        AlreadyRegistered,
        NotRegistered,
        InsufficientBalance,
        InsufficientPledge,
        NotOwner,
        InvalidOrderId,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;
    }
}

/// ledger
impl<T: Config> Module<T> {
    fn update_ledger(user: &T::AccountId, ledger: &UserLedgerOf<T>) {
        T::Currency::set_lock(
            MARKETPLACE_ID,
            &ledger.who,
            ledger.total,
            WithdrawReasons::all(),
        );

        <Users<T>>::insert(user, ledger);
    }

    /// query whether the user is registered as a automate participants
    /// return an error [AlreadyRegistered],if user is registered.
    pub fn is_registered(account: &T::AccountId) -> DispatchResult {
        if <Users<T>>::contains_key(account) {
            return Err(Error::<T>::AlreadyRegistered.into());
        }
        Ok(())
    }

    /// query whether the user is not registered as a automate participants
    /// return an error [NotRegistered], if not.
    pub fn is_not_registered(account: &T::AccountId) -> DispatchResult {
        if !<Users<T>>::contains_key(account) {
            return Err(Error::<T>::NotRegistered.into());
        }
        Ok(())
    }

    /// query if the account have enough free balance.
    fn enough_balance(account: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
        if amount > T::Currency::free_balance(&account) {
            return Err(Error::<T>::InsufficientBalance.into());
        }
        Ok(())
    }

    /// query if the account have enough active staking to use automata service.
    pub fn enough_stake(account: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
        let ledger = <Users<T>>::get(&account);
        ensure!(amount <= ledger.active, Error::<T>::InsufficientPledge);
        Ok(())
    }

    /// query if the account have enough active staking to put up an order.
    /// if not,return an error [InsufficientPledge],else lock some staking.
    pub fn enough_stake_put_order(account: &T::AccountId, pledge: BalanceOf<T>) -> DispatchResult {
        let mut ledger = <Users<T>>::get(&account);
        ensure!(ledger.active > pledge, Error::<T>::InsufficientPledge);
        ledger.active -= pledge;
        Ok(())
    }

    /// Record the order in the ledger.
    pub fn update_put_order(
        account: &T::AccountId,
        amount: BalanceOf<T>,
        order_id: OrderId<T>,
    ) -> DispatchResult {
        let mut ledger = <Users<T>>::get(&account);
        ledger.seller.insert(order_id.clone(), amount);
        <Users<T>>::insert(&account, ledger);
        Ok(())
    }

    /// register as a automata user.
    pub fn register(account: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
        Self::is_registered(&account)?;
        Self::enough_balance(&account, amount)?;

        let ledger = UserLedger {
            who: account.clone(),
            total: amount,
            active: amount,
            buyer: BTreeMap::default(),
            seller: BTreeMap::default(),
        };

        Self::update_ledger(&account, &ledger);
        Ok(())
    }

    /// register add the amount of staking.
    pub fn pledge(account: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
        Self::is_not_registered(&account)?;
        Self::enough_balance(&account, amount)?;

        let mut ledger = <Users<T>>::get(&account);
        ledger.total += amount;
        ledger.active += amount;
        Self::update_ledger(&account, &ledger);

        Ok(())
    }

    /// register reduce the amount of staking.
    pub fn withdraw(account: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
        Self::is_not_registered(&account)?;
        Self::enough_stake(&account, amount)?;

        let mut ledger = <Users<T>>::get(&account);
        ledger.total -= amount;
        ledger.active -= amount;
        if ledger.total.is_zero() {
            <Users<T>>::remove(&account);
            // Remove the lock.
            T::Currency::remove_lock(MARKETPLACE_ID, &account);
        } else {
            Self::update_ledger(&account, &ledger);
        }

        Ok(())
    }

    /// match order and exchange staking
    pub fn match_order_exchange(
        actor: T::AccountId,
        seller: T::AccountId,
        order_id: OrderId<T>,
        _duration: BlockNumber,
        cost: BalanceOf<T>,
    ) -> DispatchResult {
        let mut buyer_ledger = <Users<T>>::get(&actor);
        let mut seller_ledger = <Users<T>>::get(&seller);
        // update both ledger
        buyer_ledger.total -= cost;
        buyer_ledger.active -= cost;
        Self::update_ledger(&actor, &buyer_ledger);

        T::Currency::transfer(&actor, &seller, cost, ExistenceRequirement::AllowDeath)?;

        seller_ledger.total += cost;
        seller_ledger.active += seller_ledger.seller[&order_id];
        seller_ledger.seller.remove(&order_id);
        seller_ledger.seller.insert(order_id.clone(), cost);
        Self::update_ledger(&seller, &seller_ledger);

        // TODO transfer the right of use to buyer
        Ok(())
    }

    pub fn remove_order_or_expire_deal(seller: T::AccountId, deal_id: DealId<T>) -> DispatchResult {
        <Users<T>>::mutate(&seller, |ledger| {
            ledger.active += ledger.seller[&deal_id];
            ledger.seller.remove(&deal_id);
        });
        Ok(())
    }
}

/// slash
impl<T: Config> Module<T> {
    /// the unique hash for a report in a span of time
    fn report_sp_id<O: AutomataOffence<T::AccountId>>(
        special_id: &GeodeIdd<T::AccountId>,
        offender: &T::AccountId,
    ) -> ReportIdOf<T> {
        let unique_time = Self::time_span_filter(&special_id);
        (O::ID, unique_time.encode(), offender).using_encoded(T::Hashing::hash)
    }

    /// Divide time into spans.
    fn time_span_filter(time: &GeodeIdd<T::AccountId>) -> u64 {
        time.offline_time.clone() / 100
    }

    /// if the offence is reported, return true.
    fn duplicate(id: ReportIdOf<T>) -> bool {
        <Reports<T>>::contains_key(id)
    }

    /// compute slash for an offence
    fn compute_slash(
        balance_staking: &BalanceOf<T>,
        extra_slash_rate: &Perbill,
        reward_rate: &Perbill,
    ) -> (BalanceOf<T>, BalanceOf<T>, BalanceOf<T>) {
        let mut compensation_to_user: BalanceOf<T> = Zero::zero();
        let mut val_extra_slashed: BalanceOf<T> = Zero::zero();

        let to_extra_slash: BalanceOf<T> = *extra_slash_rate * *balance_staking;

        val_extra_slashed += to_extra_slash;

        let to_user: BalanceOf<T> = *reward_rate * to_extra_slash;
        compensation_to_user += to_user;

        let reward_payout_to_repoeter: BalanceOf<T> = val_extra_slashed - compensation_to_user;

        (
            val_extra_slashed,
            compensation_to_user,
            reward_payout_to_repoeter,
        )
    }

    /// slash the offence,reward the reporter.
    fn do_slash_reward_compensate(
        provider: &T::AccountId,
        offence_geoid: &T::AccountId,
        user: &T::AccountId,
        reporters: &[T::AccountId],
        val_slashed: BalanceOf<T>,
        val_reward: BalanceOf<T>,
        val_compensation: BalanceOf<T>,
    ) {
        //TODO:: Return fee

        // Provider slash
        let mut ledger: UserLedgerOf<T> = <Users<T>>::get(provider);
        ledger.total -= val_slashed;
        let new_stake = *ledger.seller.get(offence_geoid).unwrap() - val_slashed;
        ledger.seller.insert(offence_geoid.clone(), new_stake);
        <Users<T>>::insert(provider, ledger);

        // reword report
        //TODO:: div the val_reward
        let _num_reporter = reporters.len();
        for reporter in reporters {
            let mut ledger: UserLedgerOf<T> = <Users<T>>::get(reporter);
            ledger.total += val_reward;
            ledger.active += val_reward;
            <Users<T>>::insert(reporter, ledger);
        }

        // compensate user
        let mut ledger: UserLedgerOf<T> = <Users<T>>::get(user);
        ledger.total += val_compensation;
        ledger.active += val_compensation;
        <Users<T>>::insert(user, ledger);
    }

    /// record the offence
    fn update_report(
        unique_key: &ReportIdOf<T>,
        offender_detail: AutomataOffenceDetails<T::AccountId, T::BlockNumber>,
    ) {
        <Reports<T>>::insert(unique_key, offender_detail.clone());
    }

    /// Get his historical offences and update this offence
    fn update_offence_info(id: &GeodeId<T>) -> Option<(CurrentOffline, TotalOffline)> {
        let offence_info = <TempOffence<T>>::get(id);
        if offence_info.is_none() {
            <TempOffence<T>>::insert(id, OffenceTempRecordOf::<T>::new(id.clone()));
            return Some((1, 1));
        }
        let mut info = offence_info.unwrap();
        info.increase();
        <TempOffence<T>>::insert(id, info.clone());
        return Some((info.current_offline, info.historical_offline));
    }
}

impl<T: Config> OnAutomataOffenceHandler<T::AccountId, T::BlockNumber, Weight> for Module<T> {
    fn on_offence(
        offenders: AutomataOffenceDetails<T::AccountId, T::BlockNumber>,
        slash_fraction: Perbill,
    ) -> Result<Weight, ()> {
        let ledger: UserLedgerOf<T> = <Users<T>>::get(&offenders.provider);
        let seller_list = ledger.seller;
        let lock_bal = seller_list.get(&offenders.offender).unwrap();

        let reward_rate: Perbill = <RewardRule>::get();
        let (to_extra_slash, to_reward_reporter, compensation_to_user) =
            Self::compute_slash(lock_bal, &slash_fraction, &reward_rate);

        Self::do_slash_reward_compensate(
            &offenders.provider,
            &offenders.offender,
            &offenders.user,
            &offenders.reporters[..],
            to_extra_slash,
            to_reward_reporter,
            compensation_to_user,
        );

        Ok(0u64)
    }

    /// Determine if you can report offence now , unimplented.
    fn can_report() -> bool {
        true
    }
}

impl<T: Config, O: AutomataOffence<T::AccountId>>
    AutomataReportOffence<T::AccountId, T::AccountId, O> for Module<T>
{
    /// receive an offence report.
    fn report_offence(
        reporters: Vec<T::AccountId>,
        offence: O,
    ) -> Result<(), AutomataOffenceError> {
        if Self::can_report() {
            return Err(AutomataOffenceError::NotReportSpan.into());
        }

        let offender = offence.offender();
        let kind = offence.kind();
        let spid = offence.spid();

        let unique_key = Self::report_sp_id::<O>(&spid, &offender);

        if Self::duplicate(unique_key) {
            return Err(AutomataOffenceError::DuplicateReport.into());
        }

        let reason = match &kind {
            b"geode1234:offlin" => OffenceReason::Offline,
            b"attes1234:offlin" => OffenceReason::AttestError,
            _ => OffenceReason::Unkown,
        };

        let (current_offence, total_offence) = Self::update_offence_info(&offender).unwrap();

        let offender_detail = AutomataOffenceDetails::<T::AccountId, T::BlockNumber> {
            offender,
            reporters: reporters.clone(),
            blocknum: <frame_system::Module<T>>::block_number(),
            reason,
            provider: spid.provider,
            user: spid.user,
            offline_count: current_offence,
        };

        Self::update_report(&unique_key, offender_detail.clone());

        let extra_slash = offence.slash(current_offence, total_offence, <SlashRule>::get());

        match Self::on_offence(offender_detail, extra_slash) {
            Ok(_) => {}
            Err(_) => {} //TODO:: Save the failed slash
        }

        Ok(())
    }

    /// unimplemented
    fn is_known_offence(_offenders: &[T::AccountId]) -> bool {
        true
    }
}
