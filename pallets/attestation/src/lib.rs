#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
    use sp_std::prelude::*;
    use primitives::{BlockNumber};
    use sp_runtime::{RuntimeDebug, SaturatedConversion};
    use frame_support::{ensure};
    use sp_std::collections::btree_set::BTreeSet;

    #[cfg(feature = "std")]
    use serde::{Deserialize, Serialize};

	pub const ATTESTOR_REQUIRE: usize = 1;

    /// Geode state
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
    pub enum ReportType {
        /// Geode failed challange check
        Challenge,
        /// Geode failed service check
        Service,
        /// Default type
        Default,
    }

    impl Default for ReportType {
        fn default() -> Self {
            ReportType::Default
        }
    }

    /// The geode struct shows its status
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, Default)]
    pub struct Report<AccountId, Hash> {
        /// Report ID
        pub id: Hash,
        /// First Report from
        pub start: BlockNumber,
        pub geode: AccountId,
        pub attestors: Vec<AccountId>,
        pub report_type: ReportType,
    }
	
	pub type ReportOf<T> =
        Report<<T as frame_system::Config>::AccountId, <T as frame_system::Config>::Hash>;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_attestor::Config + pallet_geode::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	#[pallet::storage]
    #[pallet::getter(fn attestors)]
	pub(super) type Reports<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, ReportOf<T>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
        /// Attestor attestor a geode. \[attestor_id, geode_id\]
        AttestFor(T::AccountId, T::AccountId),
        /// Geodes which didn't get enough attestors at limited time after registered.
        /// \[Vec<geode_id>\]
        AttestTimeOut(Vec<T::AccountId>),
        /// Somebody report a misconduct. \[reporter, offender\]
        ReportBlame(T::AccountId, T::AccountId),
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		 /// Duplicate attestor for geode.
		 AlreadyAttestFor,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T:Config> Pallet<T> {
        /// Report that somebody did a misconduct. The actual usage is being considered.
        #[pallet::weight(0)]
        pub fn report_misconduct(origin: OriginFor<T>, geode_id: T::AccountId, _proof: Vec<u8>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // TODO
            Self::deposit_event(Event::ReportBlame(who, geode_id));
            Ok(().into())
        }

        /// Called by attestor to attest Geode.
        #[pallet::weight(0)]
        pub fn attestor_attest_geode(origin: OriginFor<T>, geode: T::AccountId) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // check attestor existance and whether atteseted
            ensure!(pallet_attestor::Attestors::<T>::contains_key(&who), pallet_attestor::Error::<T>::InvalidAttestor);
            let mut attestor = pallet_attestor::Attestors::<T>::get(&who);
            ensure!(!attestor.geodes.contains(&geode), Error::<T>::AlreadyAttestFor);

            // check geode existance and state
            ensure!(pallet_geode::Geodes::<T>::contains_key(&geode), pallet_geode::Error::<T>::InvalidGeode);
            let mut geode_record = pallet_geode::Geodes::<T>::get(&geode);
            ensure!(geode_record.state != pallet_geode::GeodeState::Unknown && geode_record.state != pallet_geode::GeodeState::Offline, 
                pallet_geode::Error::<T>::InvalidGeodeState);
                 
            // update pallet_attestor::Attestors
            attestor.geodes.insert(geode.clone());
            pallet_attestor::Attestors::<T>::insert(&who, attestor);
            
            // update pallet_attestor::GeodeAttestors
            let mut attestors = BTreeSet::<T::AccountId>::new();
            if pallet_attestor::GeodeAttestors::<T>::contains_key(&geode) {
                attestors = pallet_attestor::GeodeAttestors::<T>::get(&geode);
            }
            attestors.insert(who.clone());
            pallet_attestor::GeodeAttestors::<T>::insert(&geode, attestors);

            // first attestor attesting this geode
            if geode_record.state == pallet_geode::GeodeState::Registered {
                // update pallet_geode::Geodes
                geode_record.state = pallet_geode::GeodeState::Attested;
                pallet_geode::Geodes::<T>::insert(&geode, geode_record);

                // remove from pallet_geode::RegisteredGeodes
                pallet_geode::RegisteredGeodes::<T>::remove(&geode);

                // move into pallet_geode::AttestedGeodes
                let block_number = <frame_system::Module<T>>::block_number();
                pallet_geode::AttestedGeodes::<T>::insert(&geode, block_number.saturated_into::<BlockNumber>());
            }

            Self::deposit_event(Event::AttestFor(who, geode));
            Ok(().into())
        }
	}
}
