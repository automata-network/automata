#![cfg_attr(not(feature = "std"), no_std)]
#![feature(map_first_last)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use codec::{Decode, Encode};
    use core::convert::TryInto;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use primitives::{BlockNumber, DispatchId};
    use sp_core::H256;
    use sp_runtime::{RuntimeDebug, SaturatedConversion};

    use frame_support::{debug::native::debug, ensure};
    use sha2::{Digest, Sha256};
    use sp_std::prelude::*;

    use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};

    pub const ALLOW_DEGRADED_DISPATCH: bool = false;
    pub const MIN_ORDER_DURATION: BlockNumber = 40;

    #[cfg(feature = "std")]
    use serde::{Deserialize, Serialize};

    /// The service order struct proposed by the user
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, Default)]
    pub struct Order {
        /// Service data.
        pub binary: Vec<u8>,
        /// Service dns.
        pub dns: Vec<u8>,
        /// Service name.
        pub name: Option<String>,
        /// duration to be served, none means run endless until removed
        pub duration: BlockNumber,
        /// maximum number of geodes to serve the order
        pub geode_num: u32,
    }

    /// Geode state
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
    pub enum ServiceState {
        /// The init state when a service is created.
        Pending,
        /// When the service is being serviced by the geode.
        Online,
        // /// When the geode of the service is being reported.
        // Degraded,
        /// When the service turns offline.
        Offline,
        // /// When the service is finished
        // Completed,
        /// When the service is terminating
        Terminating,
    }

    impl Default for ServiceState {
        fn default() -> Self {
            ServiceState::Pending
        }
    }

    /// The full service struct shows its status
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, Default)]
    pub struct Service<AccountId: Ord, Hash> {
        /// Service order id.
        pub order_id: Hash,
        /// dispatch id.
        pub dispatches: BTreeSet<DispatchId>,
        /// Service creator id.
        pub owner: AccountId,
        /// Geodes being dispatched to fulfill the service.
        pub geodes: BTreeSet<AccountId>,
        /// Total block number the service has been online
        pub weighted_uptime: u64,
        /// Expected to be ended at
        pub expected_ending: Option<BlockNumber>,
        /// Whether the service has backup
        pub backup_flag: bool,
        /// Indexing for backups, key is the backup service id, value is the backup data hash
        pub backup_map: BTreeMap<AccountId, Hash>,
        /// Current state of the service
        pub state: ServiceState,
    }

    pub type ServiceOf<T> =
        Service<<T as frame_system::Config>::AccountId, <T as frame_system::Config>::Hash>;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_geode::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(block_number: T::BlockNumber) -> Weight {
            if let Ok(now) = TryInto::<BlockNumber>::try_into(block_number) {
                // process pending service orders
                {
                    // load all the promised geodes into memory
                    let mut avail_geodes = BTreeMap::<BlockNumber, Vec<T::AccountId>>::new();
                    // let mut avail_promises = Vec::<T::BlockNumber>::new();
                    let mut updated_geodes = BTreeMap::<BlockNumber, Vec<T::AccountId>>::new();
                    pallet_geode::PromisedGeodes::<T>::iter()
                        .map(|(promise, geodes)| {
                            avail_geodes.insert(promise.clone().into(), geodes);
                        })
                        .all(|_| true);

                    let mut processed_services = Vec::<u32>::new();
                    for (dispatch, order_id) in <PendingDispatches<T>>::iter() {
                        if avail_geodes.is_empty() {
                            break;
                        }

                        let order = <Orders<T>>::get(order_id);

                        let geode;

                        let expected_promise = order.duration
                            + now
                            + pallet_geode::PUT_ONLINE_TIMEOUT
                            + pallet_geode::DISPATCH_CONFIRMATION_TIMEOUT;
                        let promise;
                        if let Some(entry) = avail_geodes.range(expected_promise..).next() {
                            // try to find the smallest larger geode
                            promise = entry.0.to_owned();
                        } else if avail_geodes.contains_key(&0) {
                            promise = 0;
                        } else if ALLOW_DEGRADED_DISPATCH {
                            if let Some(entry) = avail_geodes.range(..expected_promise).last() {
                                // else find the largest smaller geode
                                promise = entry.0.to_owned();
                            } else {
                                break;
                            }
                        } else {
                            continue;
                        }

                        geode = avail_geodes.get_mut(&promise).unwrap().remove(0);
                        updated_geodes
                            .insert(promise.clone(), avail_geodes.get(&promise).unwrap().clone());

                        if avail_geodes.get(&promise).unwrap().is_empty() {
                            avail_geodes.remove(&promise);
                        }

                        // add to AwaitingDispatches
                        <AwaitingDispatches<T>>::insert(&geode, (&order_id, &now, &dispatch));
                        // remove from PendingDispatches
                        processed_services.push(dispatch);

                        Self::deposit_event(Event::DispatchQueriedGeode(dispatch, geode));
                    }
                    // handling the updated geode maps
                    for (p, v) in updated_geodes.iter() {
                        if v.is_empty() {
                            pallet_geode::PromisedGeodes::<T>::remove(p);
                        } else {
                            pallet_geode::PromisedGeodes::<T>::insert(p, v);
                        }
                    }
                    // remove processed services from PendingDispatches
                    for p in processed_services.iter() {
                        <PendingDispatches<T>>::remove(p);
                    }
                }

                // process expired dispatches awaiting for confirmation - no penalty for geode
                {
                    let mut expired = Vec::<T::AccountId>::new();
                    for (geode, (order_id, block_num, dispatch)) in <AwaitingDispatches<T>>::iter()
                    {
                        if block_num + pallet_geode::DISPATCH_CONFIRMATION_TIMEOUT < now {
                            // put the order back to PendingDispatches
                            <PendingDispatches<T>>::insert(dispatch, &order_id);
                            // detach geode to unknown state
                            <pallet_geode::Module<T>>::detach_geode(
                                pallet_geode::DetachOption::Unknown,
                                geode.clone(),
                                None,
                            )
                            .map_err(|e| {
                                debug!("{:?}", e);
                            })
                            .ok();
                            // clean from AwaitingDispatches
                            expired.push(geode);

                            Self::deposit_event(Event::NewPendingDispatch(dispatch, order_id));
                        }
                    }
                    // process expired
                    for p in expired.iter() {
                        <AwaitingDispatches<T>>::remove(p);
                    }
                }

                // process expired dispatches awaiting to be put online - have penalty for geode
                {
                    let mut expired = Vec::<T::AccountId>::new();
                    for (geode, (order_id, block_num, dispatch)) in <PreOnlineDispatches<T>>::iter()
                    {
                        if block_num + pallet_geode::PUT_ONLINE_TIMEOUT < now {
                            // put the order back to PendingDispatches
                            <PendingDispatches<T>>::insert(dispatch, &order_id);
                            // detach geode to unknown state
                            <pallet_geode::Module<T>>::detach_geode(
                                pallet_geode::DetachOption::Unknown,
                                geode.clone(),
                                None,
                            )
                            .map_err(|e| {
                                debug!("{:?}", e);
                            })
                            .ok();
                            // TODO: punish geode

                            // clean from AwaitingDispatches
                            expired.push(geode);

                            Self::deposit_event(Event::NewPendingDispatch(dispatch, order_id));
                        }
                    }
                    // process expired
                    for p in expired.iter() {
                        <PreOnlineDispatches<T>>::remove(p);
                    }
                }

                // check expected_endings and end services
            }
            0
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId", T::Hash = "Hash")]
    pub enum Event<T: Config> {
        /// User created service. \[user_id, service_hash\]
        ServiceCreated(T::AccountId, T::Hash),
        /// New dispatch created \[dispatch_id, service_hash\]
        NewPendingDispatch(DispatchId, T::Hash),
        /// Service removed. \[service_hash\]
        ServiceRemoved(T::Hash),
        /// Dispatch confirmed by geode \[dispatch_id, geode_id\]
        DispatchConfirmed(DispatchId, T::AccountId),
        /// Service turns online. \[service_hash\]
        ServiceOnline(T::Hash),
        /// Service gets degraded. \[service_hash\]
        ServiceDegraded(T::Hash),
        /// Service turns offline. \[service_hash\]
        ServiceOffline(T::Hash),
        /// Service gets completed. \[service_hash\]
        ServiceCompleted(T::Hash),
        /// Dispatch queried geode for dispatching. \[dispatch_id, geode_id\]
        DispatchQueriedGeode(DispatchId, T::AccountId),
        /// Dispatched geode put service online \[dispatch_id, geode_id\]
        DispatchPutOnline(DispatchId, T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Use an invalid service id.
        InvalidService,
        /// The ServiceState can't allow you to do something now.
        InvalidServiceState,
        /// You doesn't have the right to do what you want.
        NoRight,
        /// Not allowed to change duration for a service without indicating duration at creation
        InvalidDurationType,
        /// Insecure execution operated such as type overflow etc.
        InsecureExecution,
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn orders)]
    pub type Orders<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, Order, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn services)]
    pub type Services<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, ServiceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pending_services)]
    pub type PendingServices<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, BlockNumber, ValueQuery>;

    /// Value: the block number of when weighted_uptime updated last time
    #[pallet::storage]
    #[pallet::getter(fn online_services)]
    pub type OnlineServices<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, BlockNumber, ValueQuery>;

    // #[pallet::storage]
    // #[pallet::getter(fn degraded_services)]
    // pub type DegradedServices<T: Config> =
    //     StorageMap<_, Blake2_128Concat, T::Hash, BlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn offline_services)]
    pub type OfflineServices<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, BlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn terminating_services)]
    pub type TerminatingServices<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, BlockNumber, ValueQuery>;

    #[pallet::type_value]
    pub fn DefaultDispatchId<T: Config>() -> DispatchId {
        0
    }

    #[pallet::storage]
    #[pallet::getter(fn latest_dispatch_id)]
    pub type LatestDispatchId<T: Config> =
        StorageValue<_, DispatchId, ValueQuery, DefaultDispatchId<T>>;

    /// Dispatches haven't been assigned to any geode
    #[pallet::storage]
    #[pallet::getter(fn pending_dispatches)]
    pub type PendingDispatches<T: Config> =
        StorageMap<_, Blake2_128Concat, DispatchId, T::Hash, ValueQuery>;

    /// Dispatches waiting for geode's confirmation
    #[pallet::storage]
    #[pallet::getter(fn awaiting_dispatch)]
    pub type AwaitingDispatches<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        (T::Hash, BlockNumber, DispatchId),
        ValueQuery,
    >;

    /// Dispatches waiting for geode to put online
    #[pallet::storage]
    #[pallet::getter(fn pre_online_dispatch)]
    pub type PreOnlineDispatches<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        (T::Hash, BlockNumber, DispatchId),
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn expected_endings)]
    pub type ExpectedEndings<T: Config> =
        StorageMap<_, Blake2_128Concat, BlockNumber, BTreeSet<T::Hash>, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Called by user to create a service order.
        #[pallet::weight(0)]
        pub fn new_service(
            origin: OriginFor<T>,
            service_order: Order,
        ) -> DispatchResultWithPostInfo {
            ensure!(service_order.geode_num >= 1, Error::<T>::InvalidService);
            ensure!(
                service_order.duration >= MIN_ORDER_DURATION,
                Error::<T>::InvalidService
            );

            let who = ensure_signed(origin)?;
            let nonce = <frame_system::Module<T>>::account_nonce(&who);

            // TODO: calculate fee

            let mut data: Vec<u8> = Vec::new();
            data.extend_from_slice(&who.using_encoded(Self::to_ascii_hex));
            data.extend_from_slice(&nonce.encode().as_slice());

            let mut hasher = Sha256::new();
            hasher.update(data);
            let result = H256::from_slice(hasher.finalize().as_slice());
            let order_id: T::Hash = sp_core::hash::convert_hash(&result);

            let mut dispatch = <LatestDispatchId<T>>::get();

            let mut dispatches = BTreeSet::new();

            for _n in 1..service_order.geode_num {
                dispatch += 1;
                <PendingDispatches<T>>::insert(&dispatch, &order_id);
                dispatches.insert(dispatch.clone());
                Self::deposit_event(Event::NewPendingDispatch(dispatch, order_id));
            }

            <LatestDispatchId<T>>::put(&dispatch);

            let service = Service {
                order_id: order_id.clone(),
                dispatches: dispatches,
                owner: who.clone(),
                geodes: BTreeSet::new(),
                weighted_uptime: 0,
                expected_ending: None,
                backup_flag: false,
                backup_map: BTreeMap::new(),
                state: ServiceState::Pending,
            };

            <Orders<T>>::insert(&order_id, service_order);
            <Services<T>>::insert(&order_id, service);

            let block_number =
                <frame_system::Module<T>>::block_number().saturated_into::<BlockNumber>();
            <PendingServices<T>>::insert(&order_id, block_number);

            Self::deposit_event(Event::ServiceCreated(who, order_id.clone()));

            Ok(().into())
        }

        /// Called by user to remove a service order.
        #[pallet::weight(0)]
        pub fn remove_service(
            origin: OriginFor<T>,
            service_id: T::Hash,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                <Orders<T>>::contains_key(&service_id),
                Error::<T>::InvalidService
            );
            // TODO: Currently only when the order is in pending state, implement for other state
            ensure!(
                <PendingServices<T>>::contains_key(&service_id),
                Error::<T>::InvalidServiceState
            );
            let service = <Services<T>>::get(&service_id);
            ensure!(service.owner == who, Error::<T>::NoRight);

            for dispatch in service.dispatches.iter() {
                <PendingDispatches<T>>::remove(dispatch);
            }
            <PendingServices<T>>::remove(&service_id);
            <Services<T>>::remove(&service_id);
            <Orders<T>>::remove(&service_id);

            Self::deposit_event(Event::ServiceRemoved(service_id));
            Ok(().into())
        }

        /// Called by user to increase the duration of a service order, extended BlockNumber will be rounded up by SLOT_LENGTH
        #[pallet::weight(0)]
        pub fn extend_duration(
            origin: OriginFor<T>,
            service_id: T::Hash,
            extend: BlockNumber,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let mut service = <Services<T>>::get(&service_id);
            ensure!(service.owner == who, Error::<T>::NoRight);
            ensure!(
                service.state != ServiceState::Terminating,
                Error::<T>::InvalidServiceState
            );
            let mut order = <Orders<T>>::get(&service_id);
            // TODO: calculate fee

            order.duration = match order.duration.checked_add(extend) {
                Some(v) => v,
                None => {
                    return Err(Error::<T>::InsecureExecution.into());
                }
            };

            // TODO: update expected ending
            match service.expected_ending {
                Some(v) => {
                    let new_expected_ending = Self::get_expected_ending(
                        order.geode_num,
                        order.duration,
                        service.weighted_uptime,
                        service.geodes.len() as u32,
                    );
                    Self::update_expected_ending(service_id, Some(v), new_expected_ending);
                    service.expected_ending = Some(new_expected_ending);
                    <Services<T>>::insert(service_id, service);
                }
                None => {}
            }

            <Orders<T>>::insert(service_id, order);

            Ok(().into())
        }

        /// Called by geode to confirm an order
        #[pallet::weight(0)]
        pub fn geode_confirm_dispatching(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                <AwaitingDispatches<T>>::contains_key(&who),
                Error::<T>::NoRight
            );
            // load the dispatch info
            let (order_hash, _block_num, dispatch) = <AwaitingDispatches<T>>::get(&who);
            <PreOnlineDispatches<T>>::insert(
                &who,
                (
                    order_hash,
                    <frame_system::Module<T>>::block_number().saturated_into::<BlockNumber>(),
                    &dispatch,
                ),
            );
            <AwaitingDispatches<T>>::remove(&who);

            Self::deposit_event(Event::DispatchConfirmed(dispatch, who));

            Ok(().into())
        }

        /// Called by geode to start serving an order
        #[pallet::weight(0)]
        pub fn geode_start_serving(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                <PreOnlineDispatches<T>>::contains_key(&who),
                Error::<T>::NoRight
            );
            // load the dispatch info
            let (order_hash, _block_num, dispatch) = <PreOnlineDispatches<T>>::get(&who);

            let order_record = <Orders<T>>::get(order_hash);
            let mut service_record = <Services<T>>::get(order_hash);
            service_record.dispatches.remove(&dispatch);

            let now = <frame_system::Module<T>>::block_number().saturated_into::<BlockNumber>();

            <PreOnlineDispatches<T>>::remove(&who);

            match service_record.state {
                ServiceState::Pending => {
                    <OnlineServices<T>>::insert(order_hash, now);
                    <PendingServices<T>>::remove(&order_hash);
                    service_record.state = ServiceState::Online;
                    Self::deposit_event(Event::ServiceOnline(order_hash));
                }
                ServiceState::Offline => {
                    <OnlineServices<T>>::insert(order_hash, now);
                    <OfflineServices<T>>::remove(&order_hash);
                    service_record.state = ServiceState::Online;
                    Self::deposit_event(Event::ServiceOnline(order_hash));
                }
                ServiceState::Online => {
                    // update weighted_uptime
                    let last_update = <OnlineServices<T>>::get(order_hash);
                    let updated_weighted_uptime = Self::get_updated_weighted_uptime(
                        service_record.weighted_uptime,
                        last_update,
                        service_record.geodes.len() as u32,
                    );
                    service_record.weighted_uptime = updated_weighted_uptime;
                    <OnlineServices<T>>::insert(order_hash, now);
                }
                _ => {}
            }

            service_record.geodes.insert(who.clone());

            let new_expected_ending = Self::get_expected_ending(
                order_record.geode_num,
                order_record.duration,
                service_record.weighted_uptime,
                service_record.geodes.len() as u32,
            );
            Self::update_expected_ending(
                order_hash,
                service_record.expected_ending,
                new_expected_ending,
            );
            service_record.expected_ending = Some(new_expected_ending);

            <Services<T>>::insert(order_hash, service_record);

            Self::deposit_event(Event::DispatchPutOnline(dispatch, who));

            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        fn to_ascii_hex(data: &[u8]) -> Vec<u8> {
            let mut r = Vec::with_capacity(data.len() * 2);
            let mut push_nibble = |n| r.push(if n < 10 { b'0' + n } else { b'a' - 10 + n });
            for &b in data.iter() {
                push_nibble(b / 16);
                push_nibble(b % 16);
            }
            r
        }

        fn update_expected_ending(
            order_id: T::Hash,
            cur_expected_ending: Option<BlockNumber>,
            new_expected_ending: BlockNumber,
        ) {
            match cur_expected_ending {
                Some(v) => {
                    let mut ending_orders = <ExpectedEndings<T>>::get(v);
                    ending_orders.remove(&order_id);
                    if ending_orders.is_empty() {
                        <ExpectedEndings<T>>::remove(v);
                    } else {
                        <ExpectedEndings<T>>::insert(v, ending_orders);
                    }
                }
                None => {}
            }
            let mut ending_orders = <ExpectedEndings<T>>::get(new_expected_ending);
            ending_orders.insert(order_id);
            <ExpectedEndings<T>>::insert(new_expected_ending, ending_orders);
        }

        fn get_updated_weighted_uptime(
            prev_weighted_uptime: u64,
            last_update: BlockNumber,
            prev_serving_geode_num: u32,
        ) -> u64 {
            prev_weighted_uptime
                .checked_add(last_update as u64 * prev_serving_geode_num as u64)
                .unwrap()
        }

        fn get_expected_ending(
            geode_req: u32,
            duration: BlockNumber,
            weighted_uptime: u64,
            geode_num: u32,
        ) -> BlockNumber {
            let total_weighted_duration: u64 = geode_req as u64 * duration as u64;
            let left_weighted_duration = total_weighted_duration - weighted_uptime;
            left_weighted_duration
                .checked_div(geode_num as u64)
                .unwrap() as BlockNumber
        }
    }
}
