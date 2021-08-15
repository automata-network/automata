use automata_primitives::{AccountId, Hash};
use pallet_geode::Geode;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
    pub trait AttestorApi {
        fn attestor_list() -> Vec<(Vec<u8>, Vec<u8>)>;
    }

    pub trait GeodeApi {
        fn registered_geodes() -> Vec<Geode<AccountId, Hash>>;
        fn attested_geodes() -> Vec<Geode<AccountId, Hash>>;
        fn attestor_attested_geodes(attestor: AccountId) -> Vec<Geode<AccountId, Hash>>;
    }

    pub trait TransferApi {
        fn submit_unsigned_transaction(
            message: [u8; 68],
            signature_raw_bytes: [u8; 65]
        ) -> Result<(), ()>;
    }
}
