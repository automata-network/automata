use automata_primitives::{AccountId, Hash};
use pallet_geode::{Geode, GeodeState};
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
    pub trait AttestorApi {
        fn attestor_list() -> Vec<(Vec<u8>, Vec<u8>, u32)>;
        fn geode_attestors(geode: AccountId) -> Vec<(Vec<u8>, Vec<u8>)>;
        fn unsigned_attestor_notify_chain(message: Vec<u8>, signature_raw_bytes: [u8; 64]) -> Result<(), ()>;
    }

    pub trait GeodeApi {
        fn registered_geodes() -> Vec<Geode<AccountId, Hash>>;
        fn attested_geodes() -> Vec<Geode<AccountId, Hash>>;
        fn attestor_attested_geodes(attestor: AccountId) -> Vec<Geode<AccountId, Hash>>;
        fn geode_state(geode: AccountId) -> Option<GeodeState>;
    }

    pub trait TransferApi {
        fn submit_unsigned_transaction(
            message: [u8; 72],
            signature_raw_bytes: [u8; 65]
        ) -> Result<(), ()>;
    }
}
