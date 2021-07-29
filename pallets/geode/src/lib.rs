#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use codec::{Decode, Encode};
    use sp_runtime::{RuntimeDebug, SaturatedConversion};
    use sp_std::{prelude::*,collections::btree_map::BTreeMap};
    use frame_system::pallet_prelude::*;
    use frame_support::pallet_prelude::*;
    use frame_support::{ensure};
    use primitives::{BlockNumber};

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
    }

    #[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);
    
    #[pallet::storage]
    #[pallet::getter(fn geodes)]
	pub type Geodes<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, GeodeOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn registered_geode_ids)]
	pub type RegisteredGeodes<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn attested_geodes_ids)]
	pub type AttestedGeodes<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumber, ValueQuery>;

    #[pallet::call]
    impl<T:Config> Pallet<T> {
        /// Called by provider to register a geode. The user/attestors/state/provider will be
        /// set automatically regardless of what you set.
        #[pallet::weight(0)]
        pub fn geode_register(origin: OriginFor<T>, geode_record: GeodeOf<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let mut geode_record = geode_record;
            let geode = geode_record.id.clone();
            ensure!(!<Geodes<T>>::contains_key(&geode), Error::<T>::AlreadyGeode);

            let block_number = <frame_system::Module<T>>::block_number();
            geode_record.state = GeodeState::Registered;
            geode_record.provider = who.clone();

            <Geodes<T>>::insert(geode.clone(), geode_record);
            <RegisteredGeodes<T>>::insert(geode.clone(), block_number.saturated_into::<BlockNumber>());
            Self::deposit_event(Event::GeodeRegister(who, geode));
            Ok(().into())
        }

        /// Called by provider to remove geode .
        /// Return Ok() only when the geode's state is Registered/Attested
        #[pallet::weight(0)]
        pub fn geode_remove(origin: OriginFor<T>, geode: T::AccountId) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            if <Geodes<T>>::contains_key(&geode) {
                let geode_use = <Geodes<T>>::get(&geode);
                ensure!(geode_use.provider == who, Error::<T>::NoRight);
                ensure!(geode_use.state == GeodeState::Registered || geode_use.state == GeodeState::Attested, Error::<T>::InvalidGeodeState);
                <Geodes<T>>::remove(&geode);
                match geode_use.state {
                    GeodeState::Registered => {
                        <RegisteredGeodes<T>>::remove(&geode);
                    }
                    GeodeState::Attested => {
                        <AttestedGeodes<T>>::remove(&geode);
                        // clean record on attestors
                        for id in pallet_attestor::GeodeAttestors::<T>::get(&geode) {
                            let mut attestor = pallet_attestor::Attestors::<T>::get(&id);
                            attestor.geodes.remove(&geode);
                            pallet_attestor::Attestors::<T>::insert(&id, attestor);
                        }
                        pallet_attestor::GeodeAttestors::<T>::remove(&geode);
                    }
                    _ => {
                        // shouldn't happen
                    }
                }
            } else {
                return Err(Error::<T>::InvalidGeode.into());
            }

            Self::deposit_event(Event::GeodeRemove(geode));
            Ok(().into())
        }

        /// Called by provider to update geode properties
        #[pallet::weight(0)]
        pub fn update_geode_props(origin: OriginFor<T>, geode: T::AccountId, prop_name: Vec<u8>, prop_value: Vec<u8>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let mut geode_use = <Geodes<T>>::get(&geode);
            ensure!(geode_use.provider == who, Error::<T>::NoRight);
            geode_use.props.insert(prop_name, prop_value);
            <Geodes<T>>::insert(geode.clone(), geode_use);
            Self::deposit_event(Event::PropsUpdate(geode));
            Ok(().into())
        }

        /// Called by provider to bound dns to geode's ip.
        #[pallet::weight(0)]
        pub fn update_geode_dns(origin: OriginFor<T>, geode: T::AccountId, dns: Vec<u8>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let mut geode_use = <Geodes<T>>::get(&geode);
            ensure!(geode_use.provider == who, Error::<T>::NoRight);
            geode_use.dns = dns;
            <Geodes<T>>::insert(geode.clone(), geode_use);
            Self::deposit_event(Event::DnsUpdate(geode));
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
    }
}