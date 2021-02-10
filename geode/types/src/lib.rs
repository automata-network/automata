// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(any(feature = "sgx_enclave", feature = "substrate"), no_std)]

#[cfg(feature = "sgx_enclave")]
#[macro_use]
extern crate sgx_tstd as std;

mod aes128;
mod common;
mod eip712;
mod error;
mod remote_attestation;
mod secp256k1;
mod secp256r1;
mod sr25519;
mod utils;
mod witness;

pub use aes128::*;
pub use eip712::*;
pub use error::*;
pub use remote_attestation::*;
pub use secp256k1::*;
pub use secp256r1::*;
pub use sr25519::*;
pub use utils::*;
pub use witness::*;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_vote() {
        let voter = parse_string_h256("0x9999343212322323");
        let proposal = (0x1234567892232 as u64).into();
        let option = 1;
        //let timestamp:u64 = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let timestamp: u64 = 1612273931;

        let vote = Vote::new(voter, proposal, option, timestamp);

        let eip712_vote = vote.get_eip712_msg(86.into(), [1_u8; 20].into());
        let eip712_vote_string = serde_json::to_string(&eip712_vote).unwrap();
        let json_string = r#"{"types":{"EIP712Domain":[{"name":"name","type":"string"},{"name":"version","type":"string"},{"name":"chainId","type":"uint256"},{"name":"verifyingContract","type":"address"}],"Vote":[{"name":"voter","type":"uint256"},{"name":"proposal","type":"uint256"},{"name":"option","type":"uint32"},{"name":"timestamp","type":"uint64"}]},"domain":{"name":"Witness","version":"0.1.0","chainId":"86","verifyingContract":"0x0101010101010101010101010101010101010101"},"primaryType":"Vote","message":{"voter":"0x0000000000000000000000000000000000000000000000009999343212322323","proposal":"320255973466674","option":1,"timestamp":1612273931}}"#;

        assert_eq!(eip712_vote_string, json_string);
    }
}
