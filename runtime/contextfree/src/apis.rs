use automata_primitives::{AccountId, Hash};
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
    pub trait TransferApi {
        fn submit_unsigned_transaction(
            message: [u8; 72],
            signature_raw_bytes: [u8; 65]
        ) -> Result<(), ()>;
    }
}
