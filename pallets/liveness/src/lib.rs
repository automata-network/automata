#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use core::convert::{TryFrom, TryInto};
    use frame_support::{debug::native::debug, ensure};
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use primitives::BlockNumber;
    use sp_runtime::{Percent, RuntimeDebug, SaturatedConversion};
    use sp_std::borrow::ToOwned;
    use sp_std::collections::btree_set::BTreeSet;
    use sp_std::prelude::*;

    #[cfg(feature = "std")]
    use serde::{Deserialize, Serialize};

    pub const REPORT_APPROVAL_RATIO: Percent = Percent::from_percent(50);
    pub const REPORT_EXPIRY_BLOCK_NUMBER: BlockNumber = 10;
    pub const ATTESTATION_EXPIRY_BLOCK_NUMBER: BlockNumber = 30;
    pub const UNKNOWN_EXPIRY_BLOCK_NUMBER: BlockNumber = 5760;
    pub const DEGRADED_INSTANTIATED_EXPIRY_BLOCK_NUMBER: BlockNumber = 30;
    pub const ATTESTOR_NOTIFY_TIMEOUT_BLOCK_NUMBER: BlockNumber = 12;
    pub const DEFAULT_MIN_ATTESTOR_NUM: u32 = 1;

    /// Geode state
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
    pub enum ReportType {
        /// Geode failed challange check
        Challenge = 0x00,
        /// Geode failed service check
        Service,
        /// Default type
        Default,
    }

    impl TryFrom<u8> for ReportType {
        type Error = ();

        fn try_from(v: u8) -> Result<Self, Self::Error> {
            match v {
                x if x == ReportType::Challenge as u8 => Ok(ReportType::Challenge),
                x if x == ReportType::Service as u8 => Ok(ReportType::Service),
                x if x == ReportType::Default as u8 => Ok(ReportType::Default),
                _ => Err(()),
            }
        }
    }

    impl Default for ReportType {
        fn default() -> Self {
            ReportType::Default
        }
    }

    /// The geode struct shows its status
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, Default)]
    pub struct Report<AccountId: Ord> {
        pub start: BlockNumber,
        pub attestors: BTreeSet<AccountId>,
    }

    pub type ReportOf<T> = Report<<T as frame_system::Config>::AccountId>;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_attestor::Config + pallet_geode::Config
    {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // The pallet's runtime storage items.
    #[pallet::storage]
    #[pallet::getter(fn reports)]
    pub(super) type Reports<T: Config> =
        StorageMap<_, Blake2_128Concat, (T::AccountId, u8), ReportOf<T>, ValueQuery>;

    #[pallet::type_value]
    pub fn DefaultMinAttestorNum<T: Config>() -> u32 {
        DEFAULT_MIN_ATTESTOR_NUM
    }

    #[pallet::storage]
    #[pallet::getter(fn min_attestor_num)]
    pub type MinAttestorNum<T: Config> = StorageValue<_, u32, ValueQuery, DefaultMinAttestorNum<T>>;

    #[pallet::type_value]
    pub fn DefaultDegradeMode<T: Config>() -> bool {
        true
    }

    #[pallet::storage]
    #[pallet::getter(fn degrade_mode)]
    pub type DegradeMode<T: Config> = StorageValue<_, bool, ValueQuery, DefaultDegradeMode<T>>;

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Attestor attested a geode. \[attestor_id, geode_id\]
        AttestFor(T::AccountId, T::AccountId),
        /// Geodes which didn't get enough attestors at limited time after registered.
        /// \[Vec<geode_id>\]
        AttestTimeOut(Vec<T::AccountId>),
        /// Somebody report a misconduct. \[reporter, offender\]
        ReportBlame(T::AccountId, T::AccountId),
        /// Geode being slashed due to approval of misconduct report. \[geode_id\]
        SlashGeode(T::AccountId),
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [something, who]
        SomethingStored(u32, T::AccountId),
        /// Attestor exited
        AttestorExited(T::AccountId),
        /// Storage cleaned
        StorageCleaned,
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Duplicate attestor for geode.
        AlreadyAttestFor,
        /// Attestor not attesting this geode.
        NotAttestingFor,
        /// Invalid Report Type
        InvalidReportType,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// 1. At every block, check if a misconduct report has expired or not,
        /// if expired, clean the report.
        /// 2. At every block, check if any geode haven't get attested after an expiring block,
        /// if expired, clean the report.
        fn on_initialize(block_number: T::BlockNumber) -> Weight {
            if let Ok(now) = TryInto::<BlockNumber>::try_into(block_number) {
                // check is there a need to cancel degrade mode
                if <DegradeMode<T>>::get()
                    && pallet_attestor::AttestorNum::<T>::get() >= <MinAttestorNum<T>>::get()
                {
                    // reset all the start block num for degraded geode
                    <pallet_geode::Module<T>>::reset_degraded_block_num();
                    <DegradeMode<T>>::put(false);
                }

                // clean expired reports
                let mut expired = Vec::<(T::AccountId, u8)>::new();
                <Reports<T>>::iter()
                    .map(|(key, report)| {
                        if report.start + REPORT_EXPIRY_BLOCK_NUMBER < now {
                            expired.push(key);
                        }
                    })
                    .all(|_| true);
                for key in expired {
                    <Reports<T>>::remove(key);
                }

                // clean expired geodes
                let mut expired_geodes = Vec::<T::AccountId>::new();
                if !<DegradeMode<T>>::get() {
                    pallet_geode::RegisteredGeodes::<T>::iter()
                        .map(|(key, start)| {
                            if start + ATTESTATION_EXPIRY_BLOCK_NUMBER < now {
                                expired_geodes.push(key);
                            }
                        })
                        .all(|_| true);
                }

                // clean expired unknown geode
                pallet_geode::UnknownGeodes::<T>::iter()
                    .map(|(key, start)| {
                        if start + UNKNOWN_EXPIRY_BLOCK_NUMBER < now {
                            expired_geodes.push(key);
                        }
                    })
                    .all(|_| true);

                for key in expired_geodes {
                    <pallet_geode::Module<T>>::detach_geode(
                        pallet_geode::DetachOption::Remove,
                        key,
                        None,
                    )
                    .map_err(|e| {
                        debug!("{:?}", e);
                    })
                    .ok();
                }

                // detach DegradedInstantiated geodes
                if !<DegradeMode<T>>::get() {
                    let mut expired_degraded_geodes = Vec::<T::AccountId>::new();
                    pallet_geode::DegradedInstantiatedGeodes::<T>::iter()
                        .map(|(key, start)| {
                            if start + DEGRADED_INSTANTIATED_EXPIRY_BLOCK_NUMBER < now {
                                expired_degraded_geodes.push(key);
                            }
                        })
                        .all(|_| true);

                    for key in expired_degraded_geodes {
                        <pallet_geode::Module<T>>::detach_geode(
                            pallet_geode::DetachOption::Unknown,
                            key,
                            None,
                        )
                        .map_err(|e| {
                            debug!("{:?}", e);
                        })
                        .ok();
                    }
                }

                // clean expired attestors
                let mut expired_attestors = Vec::<T::AccountId>::new();
                pallet_attestor::AttestorLastNotify::<T>::iter()
                    .map(|(key, notify)| {
                        if notify + ATTESTOR_NOTIFY_TIMEOUT_BLOCK_NUMBER < now {
                            expired_attestors.push(key);
                        }
                    })
                    .all(|_| true);

                for key in expired_attestors {
                    Self::do_attestor_exit(&key);
                }
            }
            0
        }
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Report that somebody did a misconduct. The actual usage is being considered.
        #[pallet::weight(0)]
        pub fn report_misconduct(
            origin: OriginFor<T>,
            geode_id: T::AccountId,
            report_type: u8,
            _proof: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // check attestor existance and whether attested
            ensure!(
                pallet_attestor::Attestors::<T>::contains_key(&who),
                pallet_attestor::Error::<T>::InvalidAttestor
            );
            ensure!(
                pallet_attestor::Attestors::<T>::get(&who)
                    .geodes
                    .contains(&geode_id),
                Error::<T>::NotAttestingFor
            );
            // check have report
            match ReportType::try_from(report_type) {
                Ok(_) => {}
                Err(_) => {
                    return Err(Error::<T>::InvalidReportType.into());
                }
            };
            let key = (geode_id.clone(), report_type);
            let mut report = ReportOf::<T>::default();
            if <Reports<T>>::contains_key(&key) {
                report = <Reports<T>>::get(&key);
                report.attestors.insert(who.clone());
            } else {
                report.attestors.insert(who.clone());
                let block_number = <frame_system::Module<T>>::block_number();
                report.start = block_number.saturated_into::<BlockNumber>();
            }

            // check current amount of misconduct satisfying the approval ratio
            if Percent::from_rational_approximation(
                report.attestors.len(),
                pallet_attestor::GeodeAttestors::<T>::get(&geode_id).len(),
            ) >= REPORT_APPROVAL_RATIO
            {
                // slash the geode
                Self::slash_geode(&key.0);
                <Reports<T>>::remove(&key);
                Self::deposit_event(Event::SlashGeode(key.0.clone()));
            } else {
                // update report storage
                <Reports<T>>::insert(&key, report);
            }

            Self::deposit_event(Event::ReportBlame(who, key.0));
            Ok(().into())
        }

        /// Called by attestor to attest Geode.
        #[pallet::weight(0)]
        pub fn attestor_attest_geode(
            origin: OriginFor<T>,
            geode: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // check attestor existance and whether atteseted
            ensure!(
                pallet_attestor::Attestors::<T>::contains_key(&who),
                pallet_attestor::Error::<T>::InvalidAttestor
            );
            let mut attestor = pallet_attestor::Attestors::<T>::get(&who);
            ensure!(
                !attestor.geodes.contains(&geode),
                Error::<T>::AlreadyAttestFor
            );

            // check geode existance and state
            ensure!(
                pallet_geode::Geodes::<T>::contains_key(&geode),
                pallet_geode::Error::<T>::InvalidGeode
            );
            let mut geode_record = pallet_geode::Geodes::<T>::get(&geode);
            ensure!(
                geode_record.state != pallet_geode::GeodeState::Unknown
                    && geode_record.state != pallet_geode::GeodeState::Offline,
                pallet_geode::Error::<T>::InvalidGeodeState
            );

            // update pallet_attestor::Attestors
            attestor.geodes.insert(geode.clone());
            pallet_attestor::Attestors::<T>::insert(&who, attestor);

            // update pallet_attestor::GeodeAttestors
            let mut attestors = BTreeSet::<T::AccountId>::new();
            if pallet_attestor::GeodeAttestors::<T>::contains_key(&geode) {
                attestors = pallet_attestor::GeodeAttestors::<T>::get(&geode);
            }
            attestors.insert(who.clone());
            pallet_attestor::GeodeAttestors::<T>::insert(&geode, &attestors);

            // first attestor attesting this geode
            if geode_record.state == pallet_geode::GeodeState::Registered
                && attestors.len() as u32 >= <MinAttestorNum<T>>::get()
            {
                // update pallet_geode::Geodes
                geode_record.state = pallet_geode::GeodeState::Attested;
                pallet_geode::Geodes::<T>::insert(&geode, geode_record);

                // remove from pallet_geode::RegisteredGeodes
                pallet_geode::RegisteredGeodes::<T>::remove(&geode);

                // move into pallet_geode::AttestedGeodes
                let block_number = <frame_system::Module<T>>::block_number();
                pallet_geode::AttestedGeodes::<T>::insert(
                    &geode,
                    block_number.saturated_into::<BlockNumber>(),
                );
            }

            pallet_geode::GeodeUpdateCounters::<T>::insert(
                &geode,
                pallet_geode::GeodeUpdateCounters::<T>::get(&geode) + 1,
            );

            Self::deposit_event(Event::AttestFor(who, geode));
            Ok(().into())
        }

        /// Remove attestors while unlink the related geodes.
        #[pallet::weight(0)]
        pub fn attestor_exit(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                pallet_attestor::Attestors::<T>::contains_key(&who),
                pallet_attestor::Error::<T>::InvalidAttestor
            );
            Self::do_attestor_exit(&who);
            Ok(().into())
        }

        /// Called by root to set the min stake
        #[pallet::weight(0)]
        pub fn set_min_attestor_num(origin: OriginFor<T>, num: u32) -> DispatchResultWithPostInfo {
            let _who = ensure_root(origin)?;
            <MinAttestorNum<T>>::put(num);
            // TODO: cause geode state change immediately
            Ok(().into())
        }

        /// Called by root to clean all the storage
        #[pallet::weight(0)]
        pub fn clean_all_storage(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let _who = ensure_root(origin)?;
            Self::clean_storage();
            Self::deposit_event(Event::StorageCleaned);
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Slash geode including update storage and penalty related logics
        fn slash_geode(key: &T::AccountId) {
            <pallet_geode::Module<T>>::detach_geode(
                pallet_geode::DetachOption::Unknown,
                key.to_owned(),
                None,
            )
            .map_err(|e| {
                debug!("{:?}", e);
            })
            .ok();

            // TODO... Service related logic
            // TODO... Penalty related logic
        }

        /// Remove attestors while unlink the related geodes.
        pub fn do_attestor_exit(key: &T::AccountId) {
            let related_geodes = <pallet_attestor::Module<T>>::attestor_remove(key.to_owned());

            for geode in related_geodes.iter() {
                let mut attestors = pallet_attestor::GeodeAttestors::<T>::get(&geode);
                attestors.remove(&key);

                if attestors.is_empty() {
                    pallet_attestor::GeodeAttestors::<T>::remove(&geode);
                } else {
                    pallet_attestor::GeodeAttestors::<T>::insert(&geode, &attestors);
                }

                if <MinAttestorNum<T>>::get() > attestors.len() as u32 {
                    <pallet_geode::Module<T>>::degrade_geode(geode);
                } else {
                    // because GeodeUpdateCounters will be updated in degrade_geode
                    pallet_geode::GeodeUpdateCounters::<T>::insert(
                        &geode,
                        pallet_geode::GeodeUpdateCounters::<T>::get(&geode) + 1,
                    );
                }
            }
        }

        /// clean all the storage, USE WITH CARE!
        pub fn clean_storage() {
            // clean Reports
            {
                let mut reports = Vec::new();
                <Reports<T>>::iter()
                    .map(|(key, _)| {
                        reports.push(key);
                    })
                    .all(|_| true);
                for report in reports.iter() {
                    <Reports<T>>::remove(report);
                }
            }

            // reset MinAttestorNum
            <MinAttestorNum<T>>::put(DEFAULT_MIN_ATTESTOR_NUM);

            // reset DegradeMode
            <DegradeMode<T>>::put(true);

            <pallet_geode::Module<T>>::clean_storage();

            <pallet_attestor::Module<T>>::clean_storage();
        }
    }
}
