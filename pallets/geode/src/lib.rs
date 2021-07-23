#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use codec::{Decode, Encode};
    use sp_runtime::RuntimeDebug;
    use sp_std::{prelude::*,collections::btree_map::BTreeMap};

    /// Geode state
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
    pub enum GeodeState {
        /// The init state when provider register the geode.
        Registered,
        /// When geode get enough attestors' attestation, it turns to Attested.
        Attested,
        /// When geode has been assigned with an order.
        InOrder,
        /// When the geode has started serving the order
        InWork,
        /// When the geode is dropping out from an order gracefully
        DroppingOrder,
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
        pub owner: AccountId,
        /// Provider id
        pub provider: AccountId,
        /// Assigned order hash
        pub order: Option<Hash>,
        /// Geode's public ip.
        pub ip: Vec<u8>,
        /// Geods's dns
        pub url: Vec<u8>,
        /// Geodes' properties
        pub props: BTreeMap<Vec<u8>, Vec<u8>>,
    }
}