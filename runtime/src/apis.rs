use automata_primitives::{AccountId, Hash};
use codec::{Decode, Encode};
use sp_core::{H160, U256};
use sp_std::vec::Vec;
use pallet_geode::Geode;

sp_api::decl_runtime_apis! {
    pub trait AttestorApi {
        fn attestor_list() -> Vec<(Vec<u8>, Vec<u8>)>;
    }

    pub trait GeodeApi {
        fn registered_geodes() -> Vec<Geode<AccountId, Hash>>;
    }
}