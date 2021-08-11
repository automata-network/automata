#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::traits::{Currency, ReservableCurrency};
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use sp_std::collections::btree_set::BTreeSet;
    use sp_std::prelude::*;

    use automata_runtime_traits::AttestorAccounting;

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
    pub type Attestors<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId, ValueQuery>;

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Attestor registered. \[attestor_id\]
        AttestorRegister(T::AccountId),
       
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Use an invalid attestor id.
        InvalidAttestor,
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
            // T::Currency::reserve(&who, limit)?;
            // <Attestors<T>>::insert(&who, attestor);
            // Self::deposit_event(Event::AttestorRegister(who));
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Return attestors' url and pubkey list for rpc.
        pub fn attestor_list() {
        }

        
    }

    impl<T: Config>  AttestorAccounting for Pallet<T> {
        type AccountId = T::AccountId;
        type Currency = T::Currency;
        fn attestor_staking(&self) -> Result<u32, u32> {
            Ok(0_u32)
        }

    }
}
