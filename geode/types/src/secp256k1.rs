// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "std")]
use serde;
#[cfg(feature = "std")]
use serde_big_array::big_array;

#[cfg(feature = "sgx_enclave")]
use serde_big_array_sgx::big_array;
#[cfg(feature = "sgx_enclave")]
use serde_sgx as serde;

#[cfg(feature = "substrate")]
use serde_big_array_substrate::big_array;
#[cfg(feature = "substrate")]
use serde_substrate as serde;

use serde::{Deserialize, Serialize};

use core::fmt;
use std::convert::TryInto;

#[cfg(feature = "substrate")]
use sp_std::prelude::*;

#[cfg(feature = "sgx_enclave")]
use sgx_tstd::prelude::v1::*;

use crate::eip712::*;

big_array! { BigArray; }

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(
    Serialize, Deserialize, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone,
)]
pub struct Secp256k1PublicKey {
    pub gx: [u8; 32],
    pub gy: [u8; 32],
}

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(
    Serialize, Deserialize, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone,
)]
pub struct Secp256k1PrivateKey {
    pub r: [u8; 32],
}

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(
    Serialize, Deserialize, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone,
)]
pub struct Secp256k1Signature {
    pub r: [u8; 32],
    pub s: [u8; 32],
}

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(
    Serialize, Deserialize, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone,
)]
pub struct Secp256k1RecoverableSignature {
    pub v: u8,
    pub r: [u8; 32],
    pub s: [u8; 32],
}

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
pub struct Secp256k1SignedMsg<T: Serialize> {
    pub msg: T,
    pub signature: Secp256k1Signature,
}

impl Secp256k1RecoverableSignature {
    // returns a byte array representing the signature
    // eth format: [r(32), s(32), v(1)] in big endian
    pub fn ethsig_to_bytestring(&self) -> String {
        // NOTE: we are writing the values here in little endian...
        // Doing a hex::encode(values) will display them in big endian form
        let mut data = [0_u8; 65];
        data[..32].copy_from_slice(&self.r);
        data[32..64].copy_from_slice(&self.s);
        data[64] = self.v;
        format!("0x{}", hex::encode(&data)).to_owned()
    }
}

#[cfg(not(feature = "substrate"))]
impl From<rust_secp256k1::recovery::RecoverableSignature> for Secp256k1RecoverableSignature {
    fn from(item: rust_secp256k1::recovery::RecoverableSignature) -> Self {
        let (recid, sig) = item.serialize_compact();
        let mut r = [0_u8; 32];
        let mut s = [0_u8; 32];
        r.copy_from_slice(&sig[..32]);
        s.copy_from_slice(&sig[32..]);
        Secp256k1RecoverableSignature {
            v: recid.to_i32().try_into().unwrap(),
            r: r,
            s: s,
        }
    }
}

#[cfg(not(feature = "substrate"))]
impl<T: Serialize + EIP712> From<&EIP712SignedMsg<T>> for Secp256k1RecoverableSignature {
    fn from(item: &EIP712SignedMsg<T>) -> Self {
        let v = item.v;
        Secp256k1RecoverableSignature {
            // EIP712 signatures have 27 added to v
            v: v - 27,
            r: item.r.to_fixed_bytes(),
            s: item.s.to_fixed_bytes(),
        }
    }
}

#[cfg(not(feature = "substrate"))]
impl From<Secp256k1RecoverableSignature> for rust_secp256k1::recovery::RecoverableSignature {
    fn from(item: Secp256k1RecoverableSignature) -> Self {
        let recid = rust_secp256k1::recovery::RecoveryId::from_i32(item.v.into()).unwrap();
        let mut sig = [0_u8; 64];
        sig[..32].copy_from_slice(&item.r);
        sig[32..].copy_from_slice(&item.s);
        rust_secp256k1::recovery::RecoverableSignature::from_compact(&sig, recid).unwrap()
    }
}

#[cfg(not(feature = "substrate"))]
impl From<rust_secp256k1::key::PublicKey> for Secp256k1PublicKey {
    fn from(item: rust_secp256k1::key::PublicKey) -> Self {
        let mut x = [0_u8; 32];
        let mut y = [0_u8; 32];

        // libsecp256k1 - secp256k1_eckey_pubkey_serialize()
        // secp256k1_fe_get_b32(&pub[1], &elem->x);
        // secp256k1_fe_get_b32(&pub[33], &elem->y);
        // pub[0] = SECP256K1_TAG_PUBKEY_UNCOMPRESSED;
        // #define SECP256K1_TAG_PUBKEY_UNCOMPRESSED 0x04
        let serialized_bytes = item.serialize_uncompressed();
        assert_eq!(serialized_bytes[0], 0x4);

        // values are in big-endian
        x.copy_from_slice(&serialized_bytes[1..=32]);
        y.copy_from_slice(&serialized_bytes[33..=64]);

        Secp256k1PublicKey { gx: x, gy: y }
    }
}

#[cfg(not(feature = "substrate"))]
impl From<rust_secp256k1::key::SecretKey> for Secp256k1PrivateKey {
    fn from(item: rust_secp256k1::key::SecretKey) -> Self {
        let prvkey_ptr = item.as_ptr();
        let prvkey_len = item.len();
        assert_eq!(prvkey_len, 32);
        let prvkey_slice = unsafe { core::slice::from_raw_parts(prvkey_ptr, prvkey_len) };
        let mut prvkey = [0_u8; 32];
        prvkey.copy_from_slice(prvkey_slice);
        Secp256k1PrivateKey { r: prvkey }
    }
}

#[cfg(not(feature = "substrate"))]
impl From<Secp256k1PrivateKey> for rust_secp256k1::key::SecretKey {
    fn from(item: Secp256k1PrivateKey) -> Self {
        rust_secp256k1::key::SecretKey::from_slice(&item.r).unwrap()
    }
}

#[cfg(not(feature = "substrate"))]
impl From<&[u8]> for Secp256k1PrivateKey {
    fn from(item: &[u8]) -> Self {
        let mut data = [0_u8; 32];
        data.copy_from_slice(item);
        Secp256k1PrivateKey { r: data }
    }
}

#[cfg(not(feature = "substrate"))]
impl From<&str> for Secp256k1PrivateKey {
    fn from(item: &str) -> Self {
        let prvkey_str = item.trim_start_matches("0x");
        let prvkey_bytes = hex::decode(prvkey_str).unwrap();
        (&prvkey_bytes[..]).into()
    }
}

#[cfg(not(feature = "substrate"))]
impl From<rust_secp256k1::Signature> for Secp256k1Signature {
    fn from(item: rust_secp256k1::Signature) -> Self {
        // secp256k1_ecdsa_signature_load(ctx, &r, &s, sig);
        // secp256k1_scalar_get_b32(&output64[0], &r);
        // secp256k1_scalar_get_b32(&output64[32], &s);
        let mut r = [0_u8; 32];
        let mut s = [0_u8; 32];

        let serialized_bytes = item.serialize_compact();

        r.copy_from_slice(&serialized_bytes[..32]);
        s.copy_from_slice(&serialized_bytes[32..]);

        Secp256k1Signature { r: r, s: s }
    }
}

#[cfg(not(feature = "substrate"))]
impl From<Secp256k1Signature> for rust_secp256k1::Signature {
    fn from(item: Secp256k1Signature) -> Self {
        let mut serialized_bytes = [0_u8; 64];
        serialized_bytes[..32].copy_from_slice(&item.r);
        serialized_bytes[32..].copy_from_slice(&item.s);

        rust_secp256k1::Signature::from_compact(&serialized_bytes).unwrap()
    }
}

impl fmt::Display for Secp256k1PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Secp256k1PrivateKey\n").unwrap();
        write!(f, "r: {}\n", hex::encode(self.r))
    }
}

impl fmt::Display for Secp256k1PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Secp256k1PublicKey\n").unwrap();
        write!(f, "gx: {}\n", hex::encode(self.gx)).unwrap();
        write!(f, "gy: {}\n", hex::encode(self.gy))
    }
}

impl fmt::Display for Secp256k1Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Secp256k1Signature\n").unwrap();
        write!(f, "r: {}\n", hex::encode(self.r)).unwrap();
        write!(f, "s: {}\n", hex::encode(self.s))
    }
}
