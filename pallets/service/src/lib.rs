#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use codec::{Decode, Encode};
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use primitives::{BlockNumber, OrderNumber};
    use sp_core::H256;
    use sp_runtime::{RuntimeDebug, SaturatedConversion};
    use sp_std::{collections::btree_map::BTreeMap, prelude::*};
    use sha2::{Sha256, Digest};
    use frame_support::ensure;

    #[cfg(feature = "std")]
    use serde::{Deserialize, Serialize};

    pub const SLOT_LENGTH: BlockNumber = 40;

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
        pub duration: Option<BlockNumber>,
        /// maximum number of geodes to serve the order
        pub geode_num: Option<u32>,
    }

    /// Geode state
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
    pub enum ServiceState {
        /// The init state when a service is created.
        Pending,
        /// When the service get dispatched with a geode.
        Dispatched,
        /// When the service is being serviced by the geode.
        Online,
        /// When the geode of the service is being reported.
        Degraded,
        /// When the service turns offline.
        Offline,
        /// When the service is finished
        Completed,
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
        /// Service creator id.
        pub owner: AccountId,
        /// Geodes being dispatched to fulfill the service.
        pub geode: Vec<AccountId>,
        /// Total block number the service has been online
        pub uptime: BlockNumber,
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
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // /// 1. At every block, check if there is any pending service order and dispatch a geode for it
        // fn on_initialze(block_number: T::BlockNumber) -> Weight {
        //     if let Ok(now) = TryInto::<BlockNumber>::try_into(block_number) {
        //         <PendingServices<T>>::iter().map(|(key, report)|) {
                    
        //         }
        //     }
        //     0
        // }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId", T::Hash = "Hash")]
    pub enum Event<T: Config> {
        /// User created service. \[user_id, service_hash\]
        ServiceCreated(T::AccountId, T::Hash),
        /// Service removed. \[service_hash\]
        ServiceRemoved(T::Hash),
        /// Service dispatched to geode. \[service_hash, geode_id\]
        ServiceDispatched(T::Hash, T::AccountId),
        /// Service turns online. \[service_hash\]
        ServiceOnline(T::Hash),
        /// Service gets degraded. \[service_hash\]
        ServiceDegraded(T::Hash),
        /// Service turns offline. \[service_hash\]
        ServiceOffline(T::Hash),
        /// Service gets completed. \[service_hash\]
        ServiceCompleted(T::Hash),
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
    pub type Orders<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, Order, ValueQuery>;

    #[pallet::type_value]
    pub fn DefaultOrderNum<T: Config>() -> OrderNumber {
        0
    }

    #[pallet::storage]
    #[pallet::getter(fn order_count)]
    pub type OrderCount<T: Config> =
        StorageValue<_, OrderNumber, ValueQuery, DefaultOrderNum<T>>;

    #[pallet::storage]
    #[pallet::getter(fn services)]
    pub type Services<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, ServiceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pending_services_queue)]
    pub type PendingServicesQueue<T: Config> =
        StorageMap<_, Blake2_128Concat, OrderNumber, T::Hash, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pending_services)]
    pub type PendingServices<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, BlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn dispatched_services)]
    pub type DispatchedServices<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, BlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn online_services)]
    pub type OnlineServices<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, BlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn degraded_services)]
    pub type DegradedServices<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, BlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn offline_services)]
    pub type OfflineServices<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, BlockNumber, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Called by user to create a service order.
        #[pallet::weight(0)]
        pub fn new_service(origin: OriginFor<T>, service_order: Order) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let nonce = <frame_system::Module<T>>::account_nonce(&who);

            // TODO: calculate fee

            let mut data: Vec<u8> = Vec::new();
            data.extend_from_slice(&who.using_encoded(Self::to_ascii_hex));
            data.extend_from_slice(b"#");
            data.extend_from_slice(&nonce.encode().as_slice());

            let mut hasher = Sha256::new();
            hasher.update(data);
            let result = H256::from_slice(hasher.finalize().as_slice());
            let order_id: T::Hash = sp_core::hash::convert_hash(&result);

            let service = Service {
                order_id: order_id.clone(),
                owner: who.clone(),
                geode: Vec::new(),
                uptime: 0,
                backup_flag: false,
                backup_map: BTreeMap::new(),
                state: ServiceState::Pending
            };

            <Orders<T>>::insert(&order_id, service_order);
            <Services<T>>::insert(&order_id, service);

            let block_number = <frame_system::Module<T>>::block_number().saturated_into::<BlockNumber>();
            <PendingServices<T>>::insert(&order_id, block_number);

            let mut counter = <OrderCount<T>>::get();
            counter += 1;
            <PendingServicesQueue<T>>::insert(&counter, &order_id);
            <OrderCount<T>>::put(counter);

            Self::deposit_event(Event::ServiceCreated(who, order_id));

            Ok(().into())
        }

        /// Called by user to remove a service order. 
        #[pallet::weight(0)]
        pub fn remove_service(origin: OriginFor<T>, service_id: T::Hash) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(<Orders<T>>::contains_key(&service_id), Error::<T>::InvalidService);
            // TODO: Currently only when the order is in pending state, implement for other state
            ensure!(<PendingServices<T>>::contains_key(&service_id), Error::<T>::InvalidServiceState);
            let service = <Services<T>>::get(&service_id);
            ensure!(service.owner == who, Error::<T>::NoRight);

            <PendingServices<T>>::remove(&service_id);
            <Services<T>>::remove(&service_id);
            <Orders<T>>::remove(&service_id);

            Self::deposit_event(Event::ServiceRemoved(service_id));
            Ok(().into())
        }

        /// Called by user to increase the duration of a service order, extended BlockNumber will be rounded up by SLOT_LENGTH
        #[pallet::weight(0)]
        pub fn extend_duration(origin: OriginFor<T>, service_id: T::Hash, extend: BlockNumber) -> DispatchResultWithPostInfo{
            let who = ensure_signed(origin)?;
            let service = <Services<T>>::get(&service_id);
            ensure!(service.owner == who, Error::<T>::NoRight);
            let mut order = <Orders<T>>::get(&service_id);
            ensure!(order.duration != None, Error::<T>::InvalidDurationType);
            let corrected_extend = match extend % SLOT_LENGTH {
                0 => extend,
                v => extend - v + SLOT_LENGTH
            };
            // TODO: calculate fee

            order.duration = match order.duration.unwrap().checked_add(corrected_extend) {
                Some(v) => Some(v),
                None => {
                    return Err(Error::<T>::InsecureExecution.into());
                }
            };

            <Orders<T>>::insert(service_id, order);

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
    }
}
