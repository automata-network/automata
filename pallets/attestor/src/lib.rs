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
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use frame_support::traits::{Currency, ReservableCurrency};

	pub const ATTESTOR_REQUIRE: usize = 1;

	/// Attestor struct
    #[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, Default)]
    pub struct Attestor {
        /// Attestor's url, geode will get it and communicate with attestor.
        pub url: Vec<u8>,
        /// Attestor's Secp256r1PublicKey
		pub pubkey: Vec<u8>,
		
	}
	
	type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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
	pub(super) type Attestors<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Attestor, ValueQuery>;
	
	#[pallet::storage]
	#[pallet::getter(fn att_stake_min)]
	pub(super) type AttStakeMin<T: Config> = StorageValue<_, BalanceOf<T>>;

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
        /// Geodes which didn't get enough attestors at limited time after registered.
        /// \[Vec<geode_id>\]
        AttestTimeOut(Vec<T::AccountId>),
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		 /// Duplicate attestor for geode.
		 AlreadyAttestFor,
		 /// Use an invalid attestor id.
		 InvalidAttestor,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T:Config> Pallet<T> {
		/// Register as an attestor.
		#[pallet::weight(0)]
        pub fn attestor_register(origin: OriginFor<T>, url: Vec<u8>, pubkey: Vec<u8>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let limit = <AttStakeMin<T>>::get().ok_or(Error::<T>::InvalidAttestor)?;
			// <pallet_stake::Module<T>>::enough_stake(&who, limit)?;
			T::Currency::reserve(&who, limit)?;

            let attestor = Attestor {
                url,
                pubkey
            };
             <Attestors<T>>::insert(who.clone(), attestor);
             Self::deposit_event(Event::AttestorRegister(who));
             Ok(().into())
		}
		
		/// Remove self from attestors.
        ///
        ///  #note
        ///
        /// Currently, we use reliable attestor which would not do misconducts.
        /// This function should be called when the attestor is not serving for any geode.
		#[pallet::weight(0)]
        pub fn attestor_remove(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(<Attestors<T>>::contains_key(&who), Error::<T>::InvalidAttestor);
            <Attestors<T>>::remove(&who);
            Self::deposit_event(Event::AttestorRemove(who));
            Ok(().into())
		}
		
		/// Called by attestor to update its url.
        #[pallet::weight(0)]
        pub fn attestor_update(origin: OriginFor<T>, data: Vec<u8>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let mut attestor = <Attestors<T>>::get(&who);
            attestor.url = data;
            <Attestors<T>>::insert(who.clone(), attestor);
            Self::deposit_event(Event::AttestorUpdate(who));
            Ok(().into())
		}
		
		/// Called by root to set the min stake
		#[pallet::weight(0)]
        pub fn set_att_stake_min(origin: OriginFor<T>, stake: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let _who = ensure_root(origin)?;
            <AttStakeMin<T>>::put(stake);
            Ok(().into())
        }
	}

	impl<T: Config> Pallet<T> {
        /// Return attestors' url and pubkey list for rpc.
        pub fn attestor_list() -> Vec<(Vec<u8>, Vec<u8>)> {
            let mut res = Vec::new();
            <Attestors<T>>::iter()
                .map(|(_, attestor)| {
                    res.push((attestor.url.clone(), attestor.pubkey));
                })
                .all(|_| true);
            res
		}
	}
}
