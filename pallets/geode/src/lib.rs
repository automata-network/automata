#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use codec::{Decode, Encode};
    use core::convert::TryInto;
    use frame_support::ensure;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use primitives::BlockNumber;
    use sp_runtime::{RuntimeDebug, SaturatedConversion};
    use sp_std::{collections::btree_map::BTreeMap, prelude::*};

    #[cfg(feature = "std")]
    use serde::{Deserialize, Serialize};

    pub const DISPATCH_CONFIRMATION_TIMEOUT: BlockNumber = 12;
    pub const PUT_ONLINE_TIMEOUT: BlockNumber = 40;
    pub const ATTESTATION_EXPIRY_BLOCK_NUMBER: BlockNumber = 30;

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
        Degraded,
        /// Not available
        Null,
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
        pub order: Option<(Hash, Option<BlockNumber>)>,
        /// Geode's public ip.
        pub ip: Vec<u8>,
        /// Geode's dns.
        pub dns: Vec<u8>,
        /// Geodes' properties
        pub props: BTreeMap<Vec<u8>, Vec<u8>>,
        /// Current state of the geode
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
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// 1. At every block, check if any promise already expired
        fn on_initialize(block_number: T::BlockNumber) -> Weight {
            if let Ok(now) = TryInto::<BlockNumber>::try_into(block_number) {
                // clean expired promised geodes
                let mut expired = Vec::<BlockNumber>::new();
                for (promise, _geodes) in <PromisedGeodes<T>>::iter() {
                    if promise != 0
                        && promise <= now + DISPATCH_CONFIRMATION_TIMEOUT + PUT_ONLINE_TIMEOUT
                    {
                        expired.push(promise);
                    } else {
                        break;
                    }
                }
                for promise in expired {
                    <PromisedGeodes<T>>::remove(promise);
                }
            }
            0
        }
    }

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
        GeodePromiseUpdate(T::AccountId, BlockNumber),
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
        /// Invalid promise
        InvalidPromise,
        /// Invalid state transition
        InvalidTransition,
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn geodes)]
    pub type Geodes<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, GeodeOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn promised_geodes)]
    pub type PromisedGeodes<T: Config> =
        StorageMap<_, Blake2_128Concat, BlockNumber, Vec<T::AccountId>, ValueQuery>;

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
    pub type DegradedGeodes<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn offline_geodes_ids)]
    pub type OfflineGeodes<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn unknown_geodes_ids)]
    pub type UnknownGeodes<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumber, ValueQuery>;

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

            if <Geodes<T>>::contains_key(&geode_record.id) {
                let mut geode = <Geodes<T>>::get(geode_record.id);
                ensure!(geode.provider == who, Error::<T>::NoRight);
                geode.order = None;
                match Self::transit_state(&geode, GeodeState::Registered) {
                    true => { return Ok(().into()); }
                    false => { return Err(Error::<T>::InvalidTransition.into()); }
                }
            } else {
                let mut geode_record = geode_record;
                let geode = geode_record.id.clone();
    
                let block_number =
                    <frame_system::Module<T>>::block_number().saturated_into::<BlockNumber>();
                geode_record.state = GeodeState::Registered;
                geode_record.provider = who.clone();
    
                <Geodes<T>>::insert(&geode, &geode_record);
                <RegisteredGeodes<T>>::insert(&geode, &block_number);
    
                Self::deposit_event(Event::GeodeRegister(who, geode));
            }


            Ok(().into())
        }

        /// Called by provider to remove geode .
        #[pallet::weight(0)]
        pub fn geode_remove(
            origin: OriginFor<T>,
            geode: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(<Geodes<T>>::contains_key(&geode), Error::<T>::InvalidGeode);
            let geode = <Geodes<T>>::get(geode);
            ensure!(geode.provider == who, Error::<T>::NoRight);

            match Self::transit_state(&geode, GeodeState::Null) {
                true => Ok(().into()),
                false => Err(Error::<T>::InvalidTransition.into()),
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
            let block_number =
                <frame_system::Module<T>>::block_number().saturated_into::<BlockNumber>();
            ensure!(
                promise == 0
                    || promise > block_number + DISPATCH_CONFIRMATION_TIMEOUT + PUT_ONLINE_TIMEOUT,
                Error::<T>::InvalidPromise
            );
            // if the geode is instantiated, promise can only be extended
            if geode_use.state == GeodeState::Instantiated
                || geode_use.state == GeodeState::Degraded
            {
                ensure!(
                    (geode_use.promise != 0 && promise > geode_use.promise) || promise == 0,
                    Error::<T>::InvalidPromise
                );
            }

            // change PromisedGeodes record if there is
            if geode_use.state == GeodeState::Attested {
                // remove old record if there is
                if geode_use.promise
                    > block_number + DISPATCH_CONFIRMATION_TIMEOUT + PUT_ONLINE_TIMEOUT
                    || geode_use.promise == 0
                {
                    let mut geodes = <PromisedGeodes<T>>::get(&geode_use.promise);
                    if let Some(pos) = geodes.iter().position(|x| *x == geode) {
                        geodes.remove(pos);
                    }

                    if geodes.is_empty() {
                        <PromisedGeodes<T>>::remove(&geode_use.promise);
                    } else {
                        <PromisedGeodes<T>>::insert(&geode_use.promise, geodes);
                    }
                }

                let mut geodes = Vec::default();
                if <PromisedGeodes<T>>::contains_key(&promise) {
                    geodes = <PromisedGeodes<T>>::get(&promise);
                }
                geodes.push(geode.clone());
                <PromisedGeodes<T>>::insert(&promise, geodes);
            }

            geode_use.promise = promise;

            <Geodes<T>>::insert(&geode, geode_use);

            Self::deposit_event(Event::GeodePromiseUpdate(geode, promise));

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

        pub fn reset_degraded_block_num() {
            let block_number =
                <frame_system::Module<T>>::block_number().saturated_into::<BlockNumber>();
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
            <DegradedGeodes<T>>::iter()
                .map(|(id, _)| {
                    degraded_instantiated_geodes.push(id);
                })
                .all(|_| true);
            for id in degraded_instantiated_geodes {
                <DegradedGeodes<T>>::insert(id, block_number);
            }
        }

        // called by geode provider
        pub fn dismiss_geode_from_service(geode: T::AccountId, when: BlockNumber) {
            // reset geode order
            let mut geode_record = <Geodes<T>>::get(&geode);
            match geode_record.state {
                GeodeState::Degraded => {
                    <DegradedGeodes<T>>::remove(&geode);
                }
                GeodeState::Instantiated => {
                    <InstantiatedGeodes<T>>::remove(&geode);
                }
                _ => {}
            }
            geode_record.state = GeodeState::Offline;
            geode_record.order = None;
            <OfflineGeodes<T>>::insert(geode, when);
        }

        fn clean_from_promises(geode: &GeodeOf<T>, when: &BlockNumber) {
            // remove PromisedGeode record if there is
            if geode.promise > when + DISPATCH_CONFIRMATION_TIMEOUT + PUT_ONLINE_TIMEOUT
                || geode.promise == 0
            {
                let mut geodes = <PromisedGeodes<T>>::get(&geode.promise);
                if let Some(pos) = geodes.iter().position(|x| *x == geode.id) {
                    geodes.remove(pos);
                }

                if geodes.is_empty() {
                    <PromisedGeodes<T>>::remove(&geode.promise);
                } else {
                    <PromisedGeodes<T>>::insert(&geode.promise, geodes);
                }
            }
        }

        pub fn add_to_promises(geode: &GeodeOf<T>, when: &BlockNumber) {
            // move into the PromisedGeodes for queueing for job
            if geode.promise > when + DISPATCH_CONFIRMATION_TIMEOUT + PUT_ONLINE_TIMEOUT
                || geode.promise == 0
            {
                let mut promised_geodes = PromisedGeodes::<T>::get(&geode.promise);
                promised_geodes.push(geode.id.clone());
                PromisedGeodes::<T>::insert(geode.promise, &promised_geodes);
            }
        }

        pub fn transit_state(geode: &GeodeOf<T>, to: GeodeState) -> bool {
            let when = <frame_system::Module<T>>::block_number().saturated_into::<BlockNumber>();
            match geode.state {
                GeodeState::Registered => {
                    match to {
                        GeodeState::Attested => {
                            Self::add_to_promises(&geode, &when);
                        }
                        GeodeState::Unknown => {}
                        GeodeState::Offline => {}
                        GeodeState::Null => {}
                        _ => {
                            return false;
                        }
                    }
                    <RegisteredGeodes<T>>::remove(&geode.id);
                }
                GeodeState::Attested => {
                    match to {
                        GeodeState::Instantiated => {}
                        GeodeState::Registered => {
                            Self::clean_from_promises(&geode, &when);
                        }
                        GeodeState::Unknown => {
                            Self::clean_from_promises(&geode, &when);
                            pallet_attestor::Module::<T>::detach_geode_from_attestors(&geode.id);
                        }
                        GeodeState::Offline => {
                            Self::clean_from_promises(&geode, &when);
                            pallet_attestor::Module::<T>::detach_geode_from_attestors(&geode.id);
                        }
                        GeodeState::Null => {
                            Self::clean_from_promises(&geode, &when);
                            pallet_attestor::Module::<T>::detach_geode_from_attestors(&geode.id);
                        }
                        _ => {
                            return false;
                        }
                    }
                    <AttestedGeodes<T>>::remove(&geode.id);
                }
                GeodeState::Unknown => {
                    match to {
                        GeodeState::Null => {}
                        _ => {
                            return false;
                        }
                    }
                    <UnknownGeodes<T>>::remove(&geode.id);
                }
                GeodeState::Instantiated => {
                    match to {
                        GeodeState::Degraded => {}
                        GeodeState::Attested => {}
                        GeodeState::Unknown => {
                            pallet_attestor::Module::<T>::detach_geode_from_attestors(&geode.id);
                        }
                        GeodeState::Offline => {
                            pallet_attestor::Module::<T>::detach_geode_from_attestors(&geode.id);
                        }
                        _ => {
                            return false;
                        }
                    }
                    <InstantiatedGeodes<T>>::remove(&geode.id);
                }
                GeodeState::Degraded => {
                    match to {
                        GeodeState::Instantiated => {}
                        GeodeState::Registered => {}
                        GeodeState::Unknown => {
                            pallet_attestor::Module::<T>::detach_geode_from_attestors(&geode.id);
                        }
                        GeodeState::Offline => {
                            pallet_attestor::Module::<T>::detach_geode_from_attestors(&geode.id);
                        }
                        _ => {
                            return false;
                        }
                    }
                    <DegradedGeodes<T>>::remove(&geode.id);
                }
                GeodeState::Offline => {
                    match to {
                        GeodeState::Registered => {}
                        GeodeState::Null => {}
                        _ => {
                            return false;
                        }
                    }
                    <OfflineGeodes<T>>::remove(&geode.id);
                }
                GeodeState::Null => {
                    // shouldn't happen
                }
            }

            // change geode state and deposit event
            if to != GeodeState::Null {
                let mut geode = geode.clone();
                geode.state = to.clone();
                <Geodes<T>>::insert(&geode.id, &geode);
                Self::deposit_event(Event::GeodeStateUpdate(geode.id, to.clone()));
            } else {
                <Geodes<T>>::remove(&geode.id);
                Self::deposit_event(Event::GeodeRemove(geode.id.clone()));
            }

            // populate new state map
            match to {
                GeodeState::Registered => {
                    <RegisteredGeodes<T>>::insert(&geode.id, &when);
                }
                GeodeState::Attested => {
                    <AttestedGeodes<T>>::insert(&geode.id, &when);
                }
                GeodeState::Unknown => {
                    <UnknownGeodes<T>>::insert(&geode.id, &when);
                }
                GeodeState::Instantiated => {
                    <InstantiatedGeodes<T>>::insert(&geode.id, &when);
                }
                GeodeState::Degraded => {
                    <DegradedGeodes<T>>::insert(&geode.id, &when);
                }
                GeodeState::Offline => {
                    <OfflineGeodes<T>>::insert(&geode.id, &when);
                }
                _ => {}
            }

            true
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

            // clean DegradedGeodes
            {
                let mut degraded_instantiated_geodes = Vec::new();
                <DegradedGeodes<T>>::iter()
                    .map(|(key, _)| {
                        degraded_instantiated_geodes.push(key);
                    })
                    .all(|_| true);
                for degraded_instantiated_geode in degraded_instantiated_geodes.iter() {
                    <DegradedGeodes<T>>::remove(degraded_instantiated_geode);
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
        }
    }
}
