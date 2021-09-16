#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use core::convert::{TryFrom, TryInto};
    use frame_support::{debug::debug, ensure};
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use primitives::BlockNumber;
    use sp_runtime::{Percent, RuntimeDebug, SaturatedConversion};
    use sp_std::borrow::ToOwned;
    use sp_std::collections::btree_set::BTreeSet;
    use sp_std::prelude::*;

    #[cfg(feature = "std")]
    use serde::{Deserialize, Serialize};

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
        frame_system::Config
        + pallet_attestor::Config
        + pallet_geode::Config
        + pallet_service::Config
    {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        #[pallet::constant]
        type ReportExpiryBlockNumber: Get<BlockNumber>;

        #[pallet::constant]
        type ReportApprovalRatio: Get<Percent>;

        #[pallet::constant]
        type UnknownExpiryBlockNumber: Get<BlockNumber>;

        #[pallet::constant]
        type DegradedInstantiatedExpiryBlockNumber: Get<BlockNumber>;

        #[pallet::constant]
        type AttestorNotifyTimeoutBlockNumber: Get<BlockNumber>;

        #[pallet::constant]
        type DefaultMinAttestorNum: Get<u32>;
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
        T::DefaultMinAttestorNum::get()
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
        /// Invalid Input
        InvalidInput,
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
                {
                    if <DegradeMode<T>>::get()
                        && pallet_attestor::AttestorNum::<T>::get() >= <MinAttestorNum<T>>::get()
                    {
                        // reset all the start block num for degraded geode
                        <pallet_geode::Module<T>>::reset_degraded_block_num();
                        <DegradeMode<T>>::put(false);
                    }
                }

                // clean expired reports
                {
                    let mut expired = Vec::<(T::AccountId, u8)>::new();
                    <Reports<T>>::iter()
                        .map(|(key, report)| {
                            if (report.start + T::ReportExpiryBlockNumber::get()) < now {
                                expired.push(key);
                            }
                        })
                        .all(|_| true);
                    for key in expired {
                        <Reports<T>>::remove(key);
                    }
                }

                // clean expired geodes
                {
                    let mut expired_geodes = Vec::<T::AccountId>::new();
                    if !<DegradeMode<T>>::get() {
                        pallet_geode::RegisteredGeodes::<T>::iter()
                            .map(|(key, start)| {
                                if start + T::AttestationExpiryBlockNumber::get() < now {
                                    expired_geodes.push(key);
                                }
                            })
                            .all(|_| true);
                    }

                    // clean expired unknown geode
                    pallet_geode::UnknownGeodes::<T>::iter()
                        .map(|(key, start)| {
                            if start + T::UnknownExpiryBlockNumber::get() < now {
                                expired_geodes.push(key);
                            }
                        })
                        .all(|_| true);

                    for key in expired_geodes {
                        let geode = pallet_geode::Geodes::<T>::get(key);
                        <pallet_geode::Module<T>>::transit_state(
                            &geode,
                            pallet_geode::GeodeState::Null,
                        );
                    }
                }

                // detach Degraded geodes
                {
                    if !<DegradeMode<T>>::get() {
                        let mut expired_degraded_geodes = Vec::<T::AccountId>::new();
                        pallet_geode::DegradedGeodes::<T>::iter()
                            .map(|(key, start)| {
                                if start + T::DegradedInstantiatedExpiryBlockNumber::get() < now {
                                    expired_degraded_geodes.push(key);
                                }
                            })
                            .all(|_| true);

                        for key in expired_degraded_geodes {
                            Self::slash_geode(&key)
                        }
                    }
                }

                // clean expired attestors
                {
                    let mut expired_attestors = Vec::<T::AccountId>::new();
                    pallet_attestor::AttestorLastNotify::<T>::iter()
                        .map(|(key, notify)| {
                            if notify + T::AttestorNotifyTimeoutBlockNumber::get() < now {
                                expired_attestors.push(key);
                            }
                        })
                        .all(|_| true);

                    for key in expired_attestors {
                        Self::do_attestor_exit(&key);
                    }
                }

                {
                    // clean expired promised geodes
                    let mut expired = Vec::<BlockNumber>::new();
                    for (promise, geodes) in pallet_geode::PromisedGeodes::<T>::iter() {
                        if promise != 0
                            && promise
                                <= now
                                    + T::DispatchConfirmationTimeout::get()
                                    + T::PutOnlineTimeout::get()
                        {
                            expired.push(promise);
                            // remove geode from service if there is
                            for geode in geodes {
                                let geode_record = pallet_geode::Geodes::<T>::get(geode);
                                
                                match geode_record.state {
                                    pallet_geode::GeodeState::Instantiated => {
                                        Self::detach_geode_services_dispatches(&geode_record);
                                        <pallet_geode::Module<T>>::transit_state(
                                            &geode_record,
                                            pallet_geode::GeodeState::Attested,
                                        );
                                    },
                                    pallet_geode::GeodeState::Degraded => {
                                        Self::detach_geode_services_dispatches(&geode_record);
                                        <pallet_geode::Module<T>>::transit_state(
                                            &geode_record,
                                            pallet_geode::GeodeState::Registered,
                                        );
                                    },
                                    _ => {
                                        // do nothing
                                    }
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    for promise in expired {
                        pallet_geode::PromisedGeodes::<T>::remove(promise);
                    }
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
                Ok(t) => {
                    match t {
                        ReportType::Challenge => {
                            let geode = pallet_geode::Geodes::<T>::get(&geode_id);
                            if geode.state == pallet_geode::GeodeState::Registered {
                                // just exit attesting for it
                                let mut attestors =
                                    pallet_attestor::GeodeAttestors::<T>::get(&geode_id);
                                attestors.remove(&who);

                                if attestors.is_empty() {
                                    pallet_attestor::GeodeAttestors::<T>::remove(&geode_id);
                                } else {
                                    pallet_attestor::GeodeAttestors::<T>::insert(
                                        &geode_id, &attestors,
                                    );
                                }

                                Self::deposit_event(Event::ReportBlame(who, geode_id));
                                return Ok(().into());
                            }
                            ensure!(
                                geode.state == pallet_geode::GeodeState::Attested
                                    || geode.state == pallet_geode::GeodeState::Instantiated
                                    || geode.state == pallet_geode::GeodeState::Degraded,
                                pallet_geode::Error::<T>::InvalidGeodeState
                            );
                        }
                        ReportType::Service => {
                            let geode = pallet_geode::Geodes::<T>::get(&geode_id);
                            ensure!(
                                geode.state == pallet_geode::GeodeState::Instantiated
                                    || geode.state == pallet_geode::GeodeState::Degraded,
                                pallet_geode::Error::<T>::InvalidGeodeState
                            );
                            let service_use =
                                pallet_service::Services::<T>::get(geode.order.unwrap().0);
                            ensure!(
                                service_use.geodes.contains(&geode_id),
                                pallet_service::Error::<T>::InvalidServiceState
                            );
                        }
                        _ => {
                            return Err(Error::<T>::InvalidReportType.into());
                        }
                    }
                }
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
                let block_number =
                    <frame_system::Module<T>>::block_number().saturated_into::<BlockNumber>();
                report.start = block_number;
            }

            // check current amount of misconduct satisfying the approval ratio
            if Percent::from_rational_approximation(
                report.attestors.len(),
                pallet_attestor::GeodeAttestors::<T>::get(&geode_id).len(),
            ) >= T::ReportApprovalRatio::get()
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
            let geode_record = pallet_geode::Geodes::<T>::get(&geode);
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

            // posisble state change
            if attestors.len() as u32 >= <MinAttestorNum<T>>::get() {
                match geode_record.state {
                    pallet_geode::GeodeState::Registered => {
                        <pallet_geode::Module<T>>::transit_state(
                            &geode_record,
                            pallet_geode::GeodeState::Attested,
                        );
                    }
                    pallet_geode::GeodeState::Degraded => {
                        <pallet_geode::Module<T>>::transit_state(
                            &geode_record,
                            pallet_geode::GeodeState::Instantiated,
                        );
                    }
                    _ => {}
                }
            }

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

        /// Remove geodes while unlink the related service/dispatch
        /// Called by provider to turn geode offline
        #[pallet::weight(0)]
        pub fn provider_offline_geode(
            origin: OriginFor<T>,
            geode: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                pallet_geode::Geodes::<T>::contains_key(&geode),
                pallet_geode::Error::<T>::InvalidGeode
            );
            let geode = pallet_geode::Geodes::<T>::get(geode);
            ensure!(geode.provider == who, pallet_geode::Error::<T>::NoRight);
            if geode.state == pallet_geode::GeodeState::Instantiated
                || geode.state == pallet_geode::GeodeState::Degraded
            {
                ensure!(
                    geode.promise != 0
                        && geode.promise
                            < <frame_system::Module<T>>::block_number()
                                .saturated_into::<BlockNumber>(),
                    pallet_geode::Error::<T>::InvalidPromise
                );
            }

            Self::detach_geode_services_dispatches(&geode);
            match <pallet_geode::Module<T>>::transit_state(
                &geode,
                pallet_geode::GeodeState::Offline,
            ) {
                true => Ok(().into()),
                false => Err(pallet_geode::Error::<T>::InvalidTransition.into()),
            }
        }

        /// Called by root to set the min stake
        #[pallet::weight(0)]
        pub fn set_min_attestor_num(origin: OriginFor<T>, num: u32) -> DispatchResultWithPostInfo {
            let _who = ensure_root(origin)?;
            let prev_min_att_num = <MinAttestorNum<T>>::get();
            if num > prev_min_att_num {
                let mut geodes = Vec::new();
                for (geode, _block_num) in pallet_geode::AttestedGeodes::<T>::iter() {
                    let attestors = pallet_attestor::GeodeAttestors::<T>::get(&geode);
                    if num > attestors.len() as u32 {
                        // Self::degrade_geode(&geode);
                        geodes.push(geode);
                    }
                }
                for (geode, _block_num) in pallet_geode::InstantiatedGeodes::<T>::iter() {
                    let attestors = pallet_attestor::GeodeAttestors::<T>::get(&geode);
                    if num > attestors.len() as u32 {
                        geodes.push(geode);
                    }
                }
                for geode in geodes.iter() {
                    Self::degrade_geode(&geode);
                }
            } else if num < prev_min_att_num {
                let mut geodes_use = Vec::new();
                for (geode, _block_num) in pallet_geode::RegisteredGeodes::<T>::iter() {
                    let geode_use = pallet_geode::Geodes::<T>::get(&geode);
                    geodes_use.push(geode_use);
                }
                for (geode, _block_num) in pallet_geode::DegradedGeodes::<T>::iter() {
                    let geode_use = pallet_geode::Geodes::<T>::get(&geode);
                    geodes_use.push(geode_use)
                }
                for geode_use in geodes_use.iter() {
                    match geode_use.state {
                        pallet_geode::GeodeState::Registered => {
                            <pallet_geode::Module<T>>::transit_state(
                                &geode_use,
                                pallet_geode::GeodeState::Attested,
                            );
                        }
                        pallet_geode::GeodeState::Degraded => {
                            <pallet_geode::Module<T>>::transit_state(
                                &geode_use,
                                pallet_geode::GeodeState::Instantiated,
                            );
                        }
                        _ => {}
                    }
                }
            } else {
                return Err(Error::<T>::InvalidInput.into());
            }
            <MinAttestorNum<T>>::put(num);

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
        fn detach_geode_services_dispatches(geode: &pallet_geode::GeodeOf<T>) {
            // Service related logic
            // check if any service
            if geode.order != None {
                // geode having an service
                let service_id = geode.order.unwrap().0;
                let mut service_use = pallet_service::Services::<T>::get(&service_id);
                if service_use.geodes.contains(&geode.id) {
                    // geode already serving
                    // update weighted uptime
                    // get last updated block num
                    let last_update = pallet_service::OnlineServices::<T>::get(&service_id);
                    let updated_weighted_uptime =
                        <pallet_service::Module<T>>::get_updated_weighted_uptime(
                            service_use.weighted_uptime,
                            last_update,
                            service_use.geodes.len() as u32,
                        );
                    service_use.weighted_uptime = updated_weighted_uptime;

                    // remove geode from service_use
                    service_use.geodes.remove(&geode.id);

                    let block_number =
                        <frame_system::Module<T>>::block_number().saturated_into::<BlockNumber>();

                    if service_use.geodes.len() == 0 {
                        // move to Offline state
                        match service_use.state {
                            pallet_service::ServiceState::Online => {
                                pallet_service::OnlineServices::<T>::remove(&service_id);
                            }
                            _ => {
                                // no other case for now
                            }
                        }
                        // change service state
                        service_use.state = pallet_service::ServiceState::Offline;
                        // clean expected ending
                        service_use.expected_ending = None;
                        // put service to new state map
                        // pallet_service::OfflineServices::<T>::insert(&service_id, block_number);
                    } else {
                        // update latest online record for calculating weighted uptime next time
                        pallet_service::OnlineServices::<T>::insert(&service_id, block_number);
                        let order_use = pallet_service::Orders::<T>::get(&service_id);
                        let expected_ending = <pallet_service::Module<T>>::get_expected_ending(
                            order_use.geode_num,
                            order_use.duration,
                            updated_weighted_uptime,
                            service_use.geodes.len() as u32,
                        );
                        <pallet_service::Module<T>>::update_expected_ending(
                            service_id.to_owned(),
                            service_use.expected_ending,
                            expected_ending,
                        );
                        service_use.expected_ending = Some(expected_ending);
                    }
                    // generate new dispatch
                    let mut new_dispatches =
                        <pallet_service::Module<T>>::create_dispatches(1, service_id.to_owned())
                            .unwrap();
                    service_use.dispatches.append(&mut new_dispatches);
                } else {
                    // geode installing or uninstalling service
                    if service_use.state == pallet_service::ServiceState::Terminated {
                        // uninstalling
                        // do nothing
                    } else {
                        // installing
                        // reset the old dispatch
                        let (_order_id, _block_num, dispatch) =
                            pallet_service::PreOnlineDispatches::<T>::get(&geode.id);
                        let mut dispatch_use = pallet_service::Dispatches::<T>::get(&dispatch);
                        dispatch_use.geode = None;
                        dispatch_use.state = pallet_service::DispatchState::Pending;
                        pallet_service::PreOnlineDispatches::<T>::remove(&geode.id);
                        pallet_service::PendingDispatchesQueue::<T>::insert(
                            &dispatch_use.dispatch_id,
                            &dispatch_use.service_id,
                        );
                    }
                }
                pallet_service::Services::<T>::insert(service_id, service_use);
            } else {
                // check is any dispatch on it
                // if yes reset
                if pallet_service::AwaitingDispatches::<T>::contains_key(&geode.id) {
                    let (_order_id, _block_num, dispatch) =
                        pallet_service::AwaitingDispatches::<T>::get(&geode.id);
                    let mut dispatch_use = pallet_service::Dispatches::<T>::get(&dispatch);
                    dispatch_use.geode = None;
                    dispatch_use.state = pallet_service::DispatchState::Pending;
                    pallet_service::AwaitingDispatches::<T>::remove(&geode.id);
                    pallet_service::PendingDispatchesQueue::<T>::insert(
                        &dispatch_use.dispatch_id,
                        &dispatch_use.service_id,
                    );
                }
            }
        }

        /// Slash geode including update storage and penalty related logics
        fn slash_geode(key: &T::AccountId) {
            let geode = pallet_geode::Geodes::<T>::get(&key);
            Self::detach_geode_services_dispatches(&geode);

            // TODO... Penalty related logic
            <pallet_geode::Module<T>>::transit_state(&geode, pallet_geode::GeodeState::Unknown);
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
                    Self::degrade_geode(&geode);
                }
            }
        }

        fn degrade_geode(geode: &T::AccountId) {
            let geode_use = pallet_geode::Geodes::<T>::get(&geode);
            match geode_use.state {
                pallet_geode::GeodeState::Attested => {
                    // clean any existing dispatches
                    Self::detach_geode_services_dispatches(&geode_use);
                    <pallet_geode::Module<T>>::transit_state(
                        &geode_use,
                        pallet_geode::GeodeState::Registered,
                    );
                }
                pallet_geode::GeodeState::Instantiated => {
                    // if haven't put service Online
                    let service_use =
                        pallet_service::Services::<T>::get(geode_use.order.unwrap().0);
                    if !service_use.geodes.contains(&geode) {
                        Self::detach_geode_services_dispatches(&geode_use);
                    }

                    <pallet_geode::Module<T>>::transit_state(
                        &geode_use,
                        pallet_geode::GeodeState::Degraded,
                    );
                }
                _ => {
                    // no state change
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
            <MinAttestorNum<T>>::put(T::DefaultMinAttestorNum::get());

            // reset DegradeMode
            <DegradeMode<T>>::put(true);

            <pallet_geode::Module<T>>::clean_storage();

            <pallet_attestor::Module<T>>::clean_storage();
        }
    }
}
