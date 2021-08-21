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
    use pallet_attestor;

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_attestor::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// The currency in which fees are paid and contract balances are held.
        type Currency: ReservableCurrency<Self::AccountId>;

        /// Staked token for attestor register
        type AttestorStakingAmount: Get<BalanceOf<Self>>;

        /// Total reward to attestors
        type AttestorTotalReward: Get<BalanceOf<Self>>;

        /// The percentage as basic reward, (100 - BasicRewardRatio) as commission
        type BasicRewardRatio: Get<u8>;

        /// The slot length
        type SlotLength: Get<Self::BlockNumber>;

        /// The reward for each slot
        type RewardEachSlot: Get<BalanceOf<Self>>;

    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn total_distributed_reward)]
    pub type TotalDistributedReward<T: Config> =
        StorageValue<_, BalanceOf<T>>;

    // The pallet's runtime storage items.
    #[pallet::storage]
    #[pallet::getter(fn attestors)]
    pub type Attestors<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId, ValueQuery>;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Attestor registered. \[attestor_id\]
        AttestorRegister(T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidAttestor,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(block_number: T::BlockNumber) -> Weight {
            let attestors = <pallet_attestor::Pallet<T>>::get_all_attestors();
			1000
		}
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn attestor_register(
            origin: OriginFor<T>,
            
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
          
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Return attestors' url and pubkey list for rpc.
        pub fn attestor_list() {
        }
    }

    impl<T: Config>  AttestorAccounting for Pallet<T> {
        type AccountId = <T as frame_system::Config>::AccountId;
        // type Currency = T::Currency;
        fn attestor_staking(who: T::AccountId) -> DispatchResultWithPostInfo {
            <T as Config>::Currency::reserve(&who, T::AttestorStakingAmount::get())?;
            Ok(().into())
        }
    }
}
