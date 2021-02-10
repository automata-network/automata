// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(any(feature = "sgx_enclave"), no_std)]

#[cfg(feature = "sgx_enclave")]
#[macro_use]
extern crate sgx_tstd as std;

#[cfg(feature = "untrusted")]
extern crate sgx_ucrypto;

mod aes128;
mod enclave;
mod remote_attestation;
mod secp256k1;
mod secp256r1;
mod sr25519;

pub use aes128::*;
pub use remote_attestation::*;
pub use secp256k1::*;
pub use secp256r1::*;
pub use sr25519::*;

#[cfg(feature = "sgx_enclave")]
pub use enclave::*;

#[cfg(test)]
mod test {
    use super::*;
    use geode_types::*;
    use serde_json::*;

    #[test]
    fn test_sign() {
        let prvkey_bytes =
            hex::decode("60ed0dd24087f00faea4e2b556c74ebfa2f0e705f8169733b01530ce4c619883")
                .unwrap();
        let prvkey: Secp256k1PrivateKey = prvkey_bytes.as_slice().into();
        let addr = secp256k1_public_from_private(&prvkey).unwrap();
        let acctid = eth_secp256k1_to_accountid(&addr);

        let voter = (0x9999343212322323 as u64).into();
        let proposal = (0x1234567892232 as u64).into();
        let option = 1;
        //let timestamp:u64 = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let timestamp: u64 = 1612273931;

        let vote = Vote::new(voter, proposal, option, timestamp);

        let eip712_vote = vote.get_eip712_msg(86.into(), [1_u8; 20].into());

        let signature = secp256k1_eip712_sign(&prvkey, eip712_vote).unwrap();
        let rec_sig = Secp256k1RecoverableSignature {
            v: signature.v,
            r: signature.r.into(),
            s: signature.s.into(),
        };
        let metamask_signature = "0xd925d9a6ffd1b533604db6bf4f6db37572cf5d8e4b47d95c12f92bbf6779b8ce4991b434e723375320b40693e50bc3b03caf0c6463fb50fbfcee394d458c2a751b";
        let geode_signature = rec_sig.ethsig_to_bytestring();
        assert_eq!(metamask_signature, geode_signature);
    }

    #[test]
    fn test_verify() {
        let signed_vote_json = r#"{"msg":{"types":{"EIP712Domain":[{"name":"name","type":"string"},{"name":"version","type":"string"},{"name":"chainId","type":"uint256"},{"name":"verifyingContract","type":"address"}],"Vote":[{"name":"voter","type":"uint256"},{"name":"proposal","type":"uint256"},{"name":"option","type":"uint32"},{"name":"timestamp","type":"uint64"}]},"domain":{"name":"Witness","version":"0.1.0","chainId":"86","verifyingContract":"0x0101010101010101010101010101010101010101"},"primaryType":"Vote","message":{"voter":"11067934948897989411","proposal":"320255973466674","option":1,"timestamp":1612273931}},"v":27,"r":"0xd925d9a6ffd1b533604db6bf4f6db37572cf5d8e4b47d95c12f92bbf6779b8ce","s":"0x4991b434e723375320b40693e50bc3b03caf0c6463fb50fbfcee394d458c2a75"}"#;
        let signed_vote: EIP712SignedMsg<Vote> = serde_json::from_str(signed_vote_json).unwrap();
        let pubkey = secp256k1_eip712_verify(&signed_vote).unwrap();
        let account_id = eth_secp256k1_to_accountid(&pubkey);
        let acctid_str = format!("{:?}", account_id);
        let signer = "0x7EF99B0E5bEb8ae42DbF126B40b87410a440a32a".to_lowercase();
        assert_eq!(acctid_str, signer);
    }
}
