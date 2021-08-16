#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use core::convert::{TryInto};
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
            GeodeState::Registered
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
                <PromisedGeodes<T>>::iter()
                    .map(|(promise, _geodes)| {
                        if promise != 0 && promise <= now {
                            expired.push(promise);
                        }
                    })
                    .all(|_| true);
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
        GeodeStateUpdate(T::AccountId),
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
        /// Invalid promise
        InvalidPromise,
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
            let mut geode_record = geode_record;
            let geode = geode_record.id.clone();
            ensure!(!<Geodes<T>>::contains_key(&geode), Error::<T>::AlreadyGeode);

            let block_number = <frame_system::Module<T>>::block_number().saturated_into::<BlockNumber>();
            geode_record.state = GeodeState::Registered;
            geode_record.provider = who.clone();

            <Geodes<T>>::insert(&geode, &geode_record);
            <RegisteredGeodes<T>>::insert(&geode, block_number);

            ensure!(
                geode_record.promise == 0 || geode_record.promise > block_number,
                Error::<T>::InvalidInput
            );

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
            let block_number = <frame_system::Module<T>>::block_number().saturated_into::<BlockNumber>();
            ensure!(
                promise == 0 || promise > block_number,
                Error::<T>::InvalidPromise
            );
            // if the geode is instantiated, only extension is allowed
            if geode_use.state == GeodeState::Instantiated {
                ensure!((geode_use.promise != 0 && promise > geode_use.promise) || promise == 0, Error::<T>::InvalidPromise);
            }

            // change PromisedGeodes record if there is
            if geode_use.state == GeodeState::Attested {
                // remove old record if there is
                if geode_use.promise > block_number || geode_use.promise == 0 {
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

                // adding new record
                let mut geodes = Vec::default();
                if <PromisedGeodes<T>>::contains_key(&promise) {
                    geodes = <PromisedGeodes<T>>::get(&promise);
                }
                geodes.push(geode.clone());
                <PromisedGeodes<T>>::insert(&promise, geodes);
            }

            geode_use.promise = promise;
            <Geodes<T>>::insert(&geode, &geode_use);

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

            let block_number = <frame_system::Module<T>>::block_number().saturated_into::<BlockNumber>();
            <RegisteredGeodes<T>>::insert(&geode, block_number);

            <OfflineGeodes<T>>::remove(&geode);

            Self::deposit_event(Event::GeodeStateUpdate(geode));
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
                ensure!(
                    geode_use.state == GeodeState::Registered
                        || geode_use.state == GeodeState::Attested
                        || geode_use.state == GeodeState::Unknown,
                    Error::<T>::InvalidGeodeState
                );

                let block_number = <frame_system::Module<T>>::block_number().saturated_into::<BlockNumber>();

                match option {
                    DetachOption::Remove => {
                        <Geodes<T>>::remove(&geode);
                    }
                    DetachOption::Offline => {
                        geode_use.state = GeodeState::Offline;
                        <Geodes<T>>::insert(&geode, &geode_use);
                        
                        <OfflineGeodes<T>>::insert(
                            &geode,
                            &block_number,
                        );
                    }
                    DetachOption::Unknown => {
                        geode_use.state = GeodeState::Unknown;
                        <Geodes<T>>::insert(&geode, &geode_use);
                        <UnknownGeodes<T>>::insert(
                            &geode,
                            &block_number.saturated_into::<BlockNumber>(),
                        );
                    }
                }

                match geode_use.state {
                    GeodeState::Registered => {
                        <RegisteredGeodes<T>>::remove(&geode);
                    }
                    GeodeState::Attested => {
                        <AttestedGeodes<T>>::remove(&geode);
                        // remove PromisedGeode record if there is
                        if geode_use.promise > block_number || geode_use.promise == 0 {
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
                    }
                    GeodeState::Unknown => {
                        <UnknownGeodes<T>>::remove(&geode);
                    }
                    _ => {
                        // shouldn't happen
                    }
                }

                // clean record on attestors
                if pallet_attestor::GeodeAttestors::<T>::contains_key(&geode) {
                    for id in pallet_attestor::GeodeAttestors::<T>::get(&geode) {
                        let mut attestor = pallet_attestor::Attestors::<T>::get(&id);
                        attestor.geodes.remove(&geode);
                        pallet_attestor::Attestors::<T>::insert(&id, attestor);
                    }
                    pallet_attestor::GeodeAttestors::<T>::remove(&geode);
                }
            } else {
                return Err(Error::<T>::InvalidGeode);
            }

            match option {
                DetachOption::Remove => {
                    Self::deposit_event(Event::GeodeRemove(geode));
                }
                _ => {
                    Self::deposit_event(Event::GeodeStateUpdate(geode));
                }
            }
            Ok(())
        }
    }
}
