#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::traits::{Currency, ReservableCurrency};
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use primitives::BlockNumber;
    use sp_runtime::{RuntimeDebug, SaturatedConversion};
    use sp_std::collections::btree_set::BTreeSet;
    use sp_std::prelude::*;

    /// Attestor struct
    #[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, Default)]
    pub struct Attestor<AccountId: Ord> {
        /// Attestor's url, geode will get it and communicate with attestor.
        pub url: Vec<u8>,
        /// Attestor's Secp256r1PublicKey
        pub pubkey: Vec<u8>,
        /// Geode being attested by this attestor
        pub geodes: BTreeSet<AccountId>,
    }

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub type AttestorOf<T> = Attestor<<T as frame_system::Config>::AccountId>;

    pub const DEFAULT_ATT_STAKE_MIN: primitives::Balance = 1000;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// The currency in which fees are paid and contract balances are held.
        type Currency: ReservableCurrency<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // The pallet's runtime storage items.
    #[pallet::storage]
    #[pallet::getter(fn attestors)]
    pub type Attestors<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, AttestorOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn geode_attestors)]
    pub type GeodeAttestors<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BTreeSet<T::AccountId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn attestor_last_notification)]
    pub type AttestorLastNotify<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumber, ValueQuery>;

    #[pallet::type_value]
    pub(super) fn DefaultAttStakeMin<T: Config>() -> BalanceOf<T> {
        T::Currency::minimum_balance()
    }

    #[pallet::storage]
    #[pallet::getter(fn att_stake_min)]
    pub(super) type AttStakeMin<T: Config> =
        StorageValue<_, BalanceOf<T>, ValueQuery, DefaultAttStakeMin<T>>;

    #[pallet::type_value]
    pub fn DefaultAttestorNum<T: Config>() -> u32 {
        0
    }

    #[pallet::storage]
    #[pallet::getter(fn attestor_num)]
    pub type AttestorNum<T: Config> = StorageValue<_, u32, ValueQuery, DefaultAttestorNum<T>>;

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Attestor registered. \[attestor_id\]
        AttestorRegister(T::AccountId),
        /// Attestor moved. \[attestor_id\]
        AttestorRemove(T::AccountId),
        /// Attestor's url updated. \[attestor_id\]
        AttestorUpdate(T::AccountId),
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [something, who]
        SomethingStored(u32, T::AccountId),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Use an invalid attestor id.
        InvalidAttestor,
        /// Attestor already registered
        AlreadyRegistered,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register as an attestor.
        #[pallet::weight(0)]
        pub fn attestor_register(
            origin: OriginFor<T>,
            url: Vec<u8>,
            pubkey: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                !<Attestors<T>>::contains_key(&who),
                Error::<T>::AlreadyRegistered
            );
            let limit = <AttStakeMin<T>>::get();
            T::Currency::reserve(&who, limit)?;

            let attestor = AttestorOf::<T> {
                url,
                pubkey,
                geodes: BTreeSet::new(),
            };
            <Attestors<T>>::insert(&who, attestor);

            let block_number =
                <frame_system::Module<T>>::block_number().saturated_into::<BlockNumber>();
            <AttestorLastNotify<T>>::insert(&who, block_number);

            <AttestorNum<T>>::put(<AttestorNum<T>>::get() + 1);

            Self::deposit_event(Event::AttestorRegister(who));
            Ok(().into())
        }

        /// Called by attestor to update its url.
        #[pallet::weight(0)]
        pub fn attestor_update(origin: OriginFor<T>, url: Vec<u8>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let mut attestor = <Attestors<T>>::get(&who);
            attestor.url = url;
            <Attestors<T>>::insert(&who, attestor);
            Self::deposit_event(Event::AttestorUpdate(who));
            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn attestor_notify_chain(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // check attestor existance
            ensure!(
                <Attestors::<T>>::contains_key(&who),
                Error::<T>::InvalidAttestor
            );
            let block_number =
                <frame_system::Module<T>>::block_number().saturated_into::<BlockNumber>();
            <AttestorLastNotify<T>>::insert(&who, block_number);
            Ok(().into())
        }

        /// Called by root to set the min stake
        #[pallet::weight(0)]
        pub fn set_att_stake_min(
            origin: OriginFor<T>,
            stake: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _who = ensure_root(origin)?;
            <AttStakeMin<T>>::put(stake);
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Return attestors' url and pubkey list for rpc.
        pub fn attestor_list() -> Vec<(Vec<u8>, Vec<u8>, u32)> {
            let mut res = Vec::<(Vec<u8>, Vec<u8>, u32)>::new();
            <Attestors<T>>::iter()
                .map(|(_, attestor)| {
                    res.push((
                        attestor.url.clone(),
                        attestor.pubkey,
                        attestor.geodes.len() as u32,
                    ));
                })
                .all(|_| true);
            res
        }

        /// Return list of attestors of a geode
        pub fn attestors_of_geode(geode: T::AccountId) -> Vec<(Vec<u8>, Vec<u8>)> {
            let mut res = Vec::new();
            let ids = <GeodeAttestors<T>>::get(&geode);
            ids.iter()
                .map(|id| {
                    let att = <Attestors<T>>::get(&id);
                    res.push((att.url, att.pubkey))
                })
                .all(|_| true);
            res
        }

        /// remove attestor, return degraded geodes
        pub fn attestor_remove(attestor: T::AccountId) -> Vec<T::AccountId> {
            let attestor_record = <Attestors<T>>::get(&attestor);

            let mut ret = Vec::new();

            for geode in attestor_record.geodes.into_iter() {
                ret.push(geode)
            }

            // change storage
            <AttestorNum<T>>::put(<AttestorNum<T>>::get() - 1);
            <AttestorLastNotify<T>>::remove(&attestor);
            <Attestors<T>>::remove(&attestor);

            // deposit event
            Self::deposit_event(Event::AttestorRemove(attestor));

            ret
        }

        /// detach geode from attestors
        pub fn detach_geode_from_attestors(geode: &T::AccountId) {
            // clean record on attestors
            if GeodeAttestors::<T>::contains_key(&geode) {
                for id in GeodeAttestors::<T>::get(&geode) {
                    let mut attestor = Attestors::<T>::get(&id);
                    attestor.geodes.remove(&geode);
                    Attestors::<T>::insert(&id, attestor);
                }
                GeodeAttestors::<T>::remove(&geode);
            }
        }

        /// clean all the storage, USE WITH CARE!
        pub fn clean_storage() {
            // clean Attestors
            {
                let mut attestors = Vec::new();
                <Attestors<T>>::iter()
                    .map(|(key, _)| {
                        attestors.push(key);
                    })
                    .all(|_| true);
                for attestor in attestors.iter() {
                    <Attestors<T>>::remove(attestor);
                }
            }

            // clean GeodeAttestors
            {
                let mut geode_attestors = Vec::new();
                <GeodeAttestors<T>>::iter()
                    .map(|(key, _)| {
                        geode_attestors.push(key);
                    })
                    .all(|_| true);
                for geode_attestor in geode_attestors.iter() {
                    <GeodeAttestors<T>>::remove(geode_attestor);
                }
            }

            // clean AttestorLastNotify
            {
                let mut attestor_last_notifys = Vec::new();
                <AttestorLastNotify<T>>::iter()
                    .map(|(key, _)| {
                        attestor_last_notifys.push(key);
                    })
                    .all(|_| true);
                for attestor_last_notify in attestor_last_notifys.iter() {
                    <AttestorLastNotify<T>>::remove(attestor_last_notify);
                }
            }

            // reset AttestorNum
            <AttestorNum<T>>::put(0);
        }
    }
}
