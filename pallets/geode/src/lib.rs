#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use codec::{Decode, Encode};
    use frame_support::ensure;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use primitives::BlockNumber;
    use sp_runtime::{RuntimeDebug, SaturatedConversion};
    use sp_std::{collections::btree_map::BTreeMap, prelude::*};

    #[cfg(feature = "std")]
    use serde::{Deserialize, Serialize};

    /// Geode state
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
    pub enum GeodeState {
        /// The init state when provider register the geode.
        Registered,
        /// When geode get enough attestors' attestation, it turns to Attested.
        Attested,
        /// When a geode is instantiated with an order
        Instantiated,
        /// Unknown state
        Unknown,
        /// When the geode is offline
        Offline,
        /// When a geode is instantiated but lacking of attestor attesting it
        DegradedInstantiated,
        /// Not available
        Null,
    }

    #[derive(PartialEq, Eq, Clone, RuntimeDebug)]
    pub enum DetachOption {
        /// Remove the geode
        Remove,
        /// Turn the geode into Offline state
        Offline,
        /// Turn the geode into Unknown state
        Unknown,
    }

    impl Default for GeodeState {
        fn default() -> Self {
            GeodeState::Null
        }
    }

    /// The geode struct shows its status
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, Default)]
    pub struct Geode<AccountId, Hash> {
        /// Geode id.
        pub id: AccountId,
        /// Provider id
        pub provider: AccountId,
        /// Assigned order hash
        pub order: Option<Hash>,
        /// Geode's public ip.
        pub ip: Vec<u8>,
        /// Geode's dns.
        pub dns: Vec<u8>,
        /// Geodes' properties
        pub props: BTreeMap<Vec<u8>, Vec<u8>>,
        /// Current state of the geode and the block number of since last state change
        pub state: GeodeState,
        /// promise to be online until which block
        pub promise: BlockNumber,
    }

    pub type GeodeOf<T> =
        Geode<<T as frame_system::Config>::AccountId, <T as frame_system::Config>::Hash>;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_attestor::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId")]
    pub enum Event<T: Config> {
        /// Provider register geode. \[provider_id, geode_id\]
        GeodeRegister(T::AccountId, T::AccountId),
        /// A geode is removed. \[geode_id\]
        GeodeRemove(T::AccountId),
        /// Geode's record updated. \[geode_id\]
        GeodeUpdate(T::AccountId),
        /// Geode's props updated. \[geode_id\]
        PropsUpdate(T::AccountId),
        /// Geode's props updated. \[geode_id\]
        DnsUpdate(T::AccountId),
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [something, who]
        SomethingStored(u32, T::AccountId),
        /// Geode's state updated
        GeodeStateUpdate(T::AccountId, GeodeState),
        /// Geode's promise updated
        GeodePromiseUpdate(T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Duplicate register geode.
        AlreadyGeode,
        /// Use an invalid geode id.
        InvalidGeode,
        /// The GeodeState can't allow you to do something now.
        InvalidGeodeState,
        /// You doesn't have the right to do what you want.
        NoRight,
        /// The geode is in work so you can't do this.
        GeodeInWork,
        /// The geode is in the market so you can't do this.
        GeodeInOrder,
        /// Invalid input
        InvalidInput,
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn geodes)]
    pub type Geodes<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, GeodeOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn registered_geode_ids)]
    pub type RegisteredGeodes<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn attested_geodes_ids)]
    pub type AttestedGeodes<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn instantiated_geodes_ids)]
    pub type InstantiatedGeodes<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn degraded_instantiated_geodes_ids)]
    pub type DegradedInstantiatedGeodes<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn offline_geodes_ids)]
    pub type OfflineGeodes<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn unknown_geodes_ids)]
    pub type UnknownGeodes<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn geode_update_counters)]
    pub type GeodeUpdateCounters<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Called by provider to register a geode. The user/attestors/state/provider will be
        /// set automatically regardless of what you set.
        #[pallet::weight(0)]
        pub fn provider_register_geode(
            origin: OriginFor<T>,
            geode_record: GeodeOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let mut geode_record = geode_record;
            let geode = geode_record.id.clone();
            ensure!(!<Geodes<T>>::contains_key(&geode), Error::<T>::AlreadyGeode);

            let block_number = <frame_system::Pallet<T>>::block_number();
            geode_record.state = GeodeState::Registered;
            geode_record.provider = who.clone();

            <Geodes<T>>::insert(geode.clone(), geode_record);
            <RegisteredGeodes<T>>::insert(&geode, block_number.saturated_into::<BlockNumber>());
            <GeodeUpdateCounters<T>>::insert(&geode, 0);
            Self::deposit_event(Event::GeodeRegister(who, geode));
            Ok(().into())
        }

        /// Called by provider to remove geode .
        /// Return Ok() only when the geode's state is Registered/Attested
        #[pallet::weight(0)]
        pub fn geode_remove(
            origin: OriginFor<T>,
            geode: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            match Self::detach_geode(DetachOption::Remove, geode, Some(who)) {
                Ok(_) => Ok(().into()),
                Err(e) => Err(e.into()),
            }
        }

        /// Called by provider to update geode properties
        #[pallet::weight(0)]
        pub fn update_geode_props(
            origin: OriginFor<T>,
            geode: T::AccountId,
            prop_name: Vec<u8>,
            prop_value: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let mut geode_use = <Geodes<T>>::get(&geode);
            ensure!(geode_use.provider == who, Error::<T>::NoRight);
            geode_use.props.insert(prop_name, prop_value);
            <Geodes<T>>::insert(&geode, geode_use);
            <GeodeUpdateCounters<T>>::insert(&geode, <GeodeUpdateCounters<T>>::get(&geode) + 1);
            Self::deposit_event(Event::PropsUpdate(geode));
            Ok(().into())
        }

        /// Called by provider to bound dns to geode's ip.
        #[pallet::weight(0)]
        pub fn update_geode_dns(
            origin: OriginFor<T>,
            geode: T::AccountId,
            dns: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let mut geode_use = <Geodes<T>>::get(&geode);
            ensure!(geode_use.provider == who, Error::<T>::NoRight);
            geode_use.dns = dns;
            <Geodes<T>>::insert(&geode, geode_use);
            <GeodeUpdateCounters<T>>::insert(&geode, <GeodeUpdateCounters<T>>::get(&geode) + 1);
            Self::deposit_event(Event::DnsUpdate(geode));
            Ok(().into())
        }

        /// Called by provider to set promise block number
        #[pallet::weight(0)]
        pub fn update_geode_promise(
            origin: OriginFor<T>,
            geode: T::AccountId,
            promise: BlockNumber,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let mut geode_use = <Geodes<T>>::get(&geode);
            ensure!(geode_use.provider == who, Error::<T>::NoRight);
            let block_number = <frame_system::Pallet<T>>::block_number();
            ensure!(
                promise == 0 || promise > block_number.saturated_into::<BlockNumber>(),
                Error::<T>::InvalidInput
            );
            geode_use.promise = promise;
            <Geodes<T>>::insert(&geode, geode_use);
            <GeodeUpdateCounters<T>>::insert(&geode, <GeodeUpdateCounters<T>>::get(&geode) + 1);
            Self::deposit_event(Event::GeodePromiseUpdate(geode));

            Ok(().into())
        }

        /// Called by provider to turn geode offline
        #[pallet::weight(0)]
        pub fn turn_geode_offline(
            origin: OriginFor<T>,
            geode: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            match Self::detach_geode(DetachOption::Offline, geode, Some(who)) {
                Ok(_) => Ok(().into()),
                Err(e) => Err(e.into()),
            }
        }

        /// Called by provider to turn geode online
        #[pallet::weight(0)]
        pub fn turn_geode_online(
            origin: OriginFor<T>,
            geode: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                !<OfflineGeodes<T>>::contains_key(&geode),
                Error::<T>::InvalidGeodeState
            );
            let mut geode_use = <Geodes<T>>::get(&geode);
            ensure!(geode_use.provider == who, Error::<T>::NoRight);

            geode_use.state = GeodeState::Registered;
            <Geodes<T>>::insert(&geode, &geode_use);

            let block_number = <frame_system::Pallet<T>>::block_number();
            <RegisteredGeodes<T>>::insert(&geode, block_number.saturated_into::<BlockNumber>());

            <OfflineGeodes<T>>::remove(&geode);

            <GeodeUpdateCounters<T>>::insert(&geode, <GeodeUpdateCounters<T>>::get(&geode) + 1);

            Self::deposit_event(Event::GeodeStateUpdate(geode, GeodeState::Registered));
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Return geodes in registered state
        pub fn registered_geodes() -> Vec<GeodeOf<T>> {
            let mut res = Vec::new();
            <RegisteredGeodes<T>>::iter()
                .map(|(id, _)| {
                    res.push(<Geodes<T>>::get(id));
                })
                .all(|_| true);
            res
        }

        /// Return geodes in attested state
        pub fn attested_geodes() -> Vec<GeodeOf<T>> {
            let mut res = Vec::new();
            <AttestedGeodes<T>>::iter()
                .map(|(id, _)| {
                    res.push(<Geodes<T>>::get(id));
                })
                .all(|_| true);
            res
        }

        /// Return list geode an attestor is attesting
        pub fn attestor_attested_geodes(attestor: T::AccountId) -> Vec<GeodeOf<T>> {
            let mut res = Vec::new();
            if pallet_attestor::Attestors::<T>::contains_key(&attestor) {
                for geode in pallet_attestor::Attestors::<T>::get(&attestor).geodes {
                    res.push(Geodes::<T>::get(&geode));
                }
            }
            res
        }

        pub fn geode_state(geode: T::AccountId) -> Option<GeodeState> {
            if <Geodes<T>>::contains_key(&geode) {
                Some(<Geodes<T>>::get(&geode).state)
            } else {
                None
            }
        }

        pub fn degrade_geode(geode: &T::AccountId) {
            let mut geode_record = <Geodes<T>>::get(&geode);
            let to_state;
            match geode_record.state {
                GeodeState::Attested => {
                    to_state = GeodeState::Registered;
                    <AttestedGeodes<T>>::remove(&geode);
                }
                GeodeState::Instantiated => {
                    to_state = GeodeState::DegradedInstantiated;
                    <InstantiatedGeodes<T>>::remove(&geode);
                }
                _ => {
                    return;
                }
            }
            geode_record.state = to_state.clone();
            <Geodes<T>>::insert(&geode, geode_record);
            <GeodeUpdateCounters<T>>::insert(&geode, <GeodeUpdateCounters<T>>::get(&geode) + 1);
            let block_number =
                <frame_system::Pallet<T>>::block_number().saturated_into::<BlockNumber>();
            match to_state {
                GeodeState::Registered => {
                    <RegisteredGeodes<T>>::insert(&geode, block_number);
                }
                GeodeState::DegradedInstantiated => {
                    <DegradedInstantiatedGeodes<T>>::insert(&geode, block_number);
                }
                _ => {}
            }
        }

        pub fn reset_degraded_block_num() {
            let block_number =
                <frame_system::Pallet<T>>::block_number().saturated_into::<BlockNumber>();
            // reset Registered
            let mut registered_geodes = Vec::new();
            <RegisteredGeodes<T>>::iter()
                .map(|(id, _)| {
                    registered_geodes.push(id);
                })
                .all(|_| true);
            for id in registered_geodes {
                <RegisteredGeodes<T>>::insert(id, block_number);
            }
            // reset DeprecatedInstantiated
            let mut degraded_instantiated_geodes = Vec::new();
            <DegradedInstantiatedGeodes<T>>::iter()
                .map(|(id, _)| {
                    degraded_instantiated_geodes.push(id);
                })
                .all(|_| true);
            for id in degraded_instantiated_geodes {
                <DegradedInstantiatedGeodes<T>>::insert(id, block_number);
            }
        }

        pub fn detach_geode(
            option: DetachOption,
            geode: T::AccountId,
            who: Option<T::AccountId>,
        ) -> Result<(), Error<T>> {
            if <Geodes<T>>::contains_key(&geode) {
                let mut geode_use = <Geodes<T>>::get(&geode);

                if let Some(who) = who {
                    ensure!(geode_use.provider == who, Error::<T>::NoRight)
                }

                let prev_state = geode_use.state.clone();

                match option {
                    DetachOption::Remove => {
                        ensure!(
                            geode_use.state == GeodeState::Registered
                                || geode_use.state == GeodeState::Attested
                                || geode_use.state == GeodeState::Unknown,
                            Error::<T>::InvalidGeodeState
                        );
                        <Geodes<T>>::remove(&geode);
                        Self::deposit_event(Event::GeodeRemove(geode.clone()));
                    }
                    DetachOption::Offline => {
                        ensure!(
                            geode_use.state == GeodeState::Registered
                                || geode_use.state == GeodeState::Attested,
                            Error::<T>::InvalidGeodeState
                        );
                        geode_use.state = GeodeState::Offline;
                        <Geodes<T>>::insert(&geode, &geode_use);
                        let block_number = <frame_system::Pallet<T>>::block_number();
                        <OfflineGeodes<T>>::insert(
                            &geode,
                            block_number.saturated_into::<BlockNumber>(),
                        );
                        <GeodeUpdateCounters<T>>::insert(
                            &geode,
                            <GeodeUpdateCounters<T>>::get(&geode) + 1,
                        );
                        Self::deposit_event(Event::GeodeStateUpdate(
                            geode.clone(),
                            GeodeState::Offline,
                        ));
                    }
                    DetachOption::Unknown => {
                        geode_use.state = GeodeState::Unknown;
                        <Geodes<T>>::insert(&geode, &geode_use);
                        let block_number = <frame_system::Pallet<T>>::block_number();
                        <UnknownGeodes<T>>::insert(
                            &geode,
                            block_number.saturated_into::<BlockNumber>(),
                        );
                        <GeodeUpdateCounters<T>>::insert(
                            &geode,
                            <GeodeUpdateCounters<T>>::get(&geode) + 1,
                        );
                        Self::deposit_event(Event::GeodeStateUpdate(
                            geode.clone(),
                            GeodeState::Unknown,
                        ));
                    }
                }

                match prev_state {
                    GeodeState::Registered => {
                        <RegisteredGeodes<T>>::remove(&geode);
                    }
                    GeodeState::Attested => {
                        <AttestedGeodes<T>>::remove(&geode);
                    }
                    GeodeState::Unknown => {
                        <UnknownGeodes<T>>::remove(&geode);
                    }
                    GeodeState::Instantiated => {
                        <InstantiatedGeodes<T>>::remove(&geode);
                    }
                    GeodeState::DegradedInstantiated => {
                        <DegradedInstantiatedGeodes<T>>::remove(&geode);
                    }
                    GeodeState::Offline => {
                        <OfflineGeodes<T>>::remove(&geode);
                    }
                    _ => {
                        // shouldn't happen
                    }
                }
                // clean record on attestors
                pallet_attestor::Module::<T>::detach_geode_from_attestors(&geode);
            } else {
                return Err(Error::<T>::InvalidGeode);
            }

            Ok(())
        }

        /// clean all the storage, USE WITH CARE!
        pub fn clean_storage() {
            // clean Geodes
            {
                let mut geodes = Vec::new();
                <Geodes<T>>::iter()
                    .map(|(key, _)| {
                        geodes.push(key);
                    })
                    .all(|_| true);
                for geode in geodes.iter() {
                    <Geodes<T>>::remove(geode);
                }
            }

            // clean RegisteredGeodes
            {
                let mut registered_geodes = Vec::new();
                <RegisteredGeodes<T>>::iter()
                    .map(|(key, _)| {
                        registered_geodes.push(key);
                    })
                    .all(|_| true);
                for registered_geode in registered_geodes.iter() {
                    <RegisteredGeodes<T>>::remove(registered_geode);
                }
            }

            // clean AttestedGeodes
            {
                let mut attested_geodes = Vec::new();
                <AttestedGeodes<T>>::iter()
                    .map(|(key, _)| {
                        attested_geodes.push(key);
                    })
                    .all(|_| true);
                for attested_geode in attested_geodes.iter() {
                    <AttestedGeodes<T>>::remove(attested_geode);
                }
            }

            // clean InstantiatedGeodes
            {
                let mut instantiated_geodes = Vec::new();
                <InstantiatedGeodes<T>>::iter()
                    .map(|(key, _)| {
                        instantiated_geodes.push(key);
                    })
                    .all(|_| true);
                for instantiated_geode in instantiated_geodes.iter() {
                    <InstantiatedGeodes<T>>::remove(instantiated_geode);
                }
            }

            // clean DegradedInstantiatedGeodes
            {
                let mut degraded_instantiated_geodes = Vec::new();
                <DegradedInstantiatedGeodes<T>>::iter()
                    .map(|(key, _)| {
                        degraded_instantiated_geodes.push(key);
                    })
                    .all(|_| true);
                for degraded_instantiated_geode in degraded_instantiated_geodes.iter() {
                    <DegradedInstantiatedGeodes<T>>::remove(degraded_instantiated_geode);
                }
            }

            // clean OfflineGeodes
            {
                let mut offline_geodes = Vec::new();
                <OfflineGeodes<T>>::iter()
                    .map(|(key, _)| {
                        offline_geodes.push(key);
                    })
                    .all(|_| true);
                for offline_geode in offline_geodes.iter() {
                    <OfflineGeodes<T>>::remove(offline_geode);
                }
            }

            // clean UnknownGeodes
            {
                let mut unknown_geodes = Vec::new();
                <UnknownGeodes<T>>::iter()
                    .map(|(key, _)| {
                        unknown_geodes.push(key);
                    })
                    .all(|_| true);
                for unknown_geode in unknown_geodes.iter() {
                    <UnknownGeodes<T>>::remove(unknown_geode);
                }
            }

            // clean GeodeUpdateCounters
            {
                let mut geode_update_counters = Vec::new();
                <GeodeUpdateCounters<T>>::iter()
                    .map(|(key, _)| {
                        geode_update_counters.push(key);
                    })
                    .all(|_| true);
                for geode_update_counter in geode_update_counters.iter() {
                    <GeodeUpdateCounters<T>>::remove(geode_update_counter);
                }
            }
        }
    }
}
