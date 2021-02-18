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

#[cfg(feature = "openssl_support")]
use openssl::ec::EcKey;

#[cfg(feature = "ring_support")]
use ring::agreement;
#[cfg(feature = "ring_support")]
use ring::signature::{self, Signature};

#[cfg(not(feature = "substrate"))]
use sgx_types::*;

#[cfg(feature = "substrate")]
use sp_std::prelude::*;

#[cfg(not(feature = "substrate"))]
use core::mem::transmute;

//#[cfg(not(feature = "substrate"))]
//use std::vec::Vec;

#[cfg(not(feature = "substrate"))]
use std::convert::From;

big_array! { BigArray; }

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(
    Serialize, Deserialize, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone,
)]
pub struct Secp256r1PublicKey {
    pub gx: [u8; 32],
    pub gy: [u8; 32],
}

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(
    Serialize, Deserialize, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone,
)]
pub struct Secp256r1PrivateKey {
    pub r: [u8; 32],
}

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(
    Serialize, Deserialize, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone,
)]
pub struct Secp256r1Signature {
    pub x: [u8; 32],
    pub y: [u8; 32],
}

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
pub struct Secp256r1SignedMsg<T: Serialize> {
    pub msg: T,
    pub signature: Secp256r1Signature,
}

impl Secp256r1PublicKey {
    #[cfg(not(feature = "substrate"))]
    pub fn from_sgx_ec256_public(key: &sgx_ec256_public_t) -> Secp256r1PublicKey {
        Secp256r1PublicKey {
            gx: key.gx,
            gy: key.gy,
        }
    }

    #[cfg(not(feature = "substrate"))]
    pub fn to_sgx_ec256_public(&self) -> sgx_ec256_public_t {
        sgx_ec256_public_t {
            gx: self.gx,
            gy: self.gy,
        }
    }

    pub fn to_raw_bytes(&self) -> [u8; 64] {
        let mut bytes = [0_u8; 64];
        bytes[..32].copy_from_slice(&self.gx);
        bytes[32..].copy_from_slice(&self.gy);
        bytes
    }

    #[cfg(feature = "ring_support")]
    pub fn to_ring_agreement_key(&self) -> agreement::UnparsedPublicKey<Vec<u8>> {
        let buf = self.to_ring_bytes();
        agreement::UnparsedPublicKey::new(&agreement::ECDH_P256, buf.to_vec())
    }

    #[cfg(feature = "ring_support")]
    pub fn to_ring_signature_key(&self) -> signature::UnparsedPublicKey<Vec<u8>> {
        let buf = self.to_ring_bytes();
        signature::UnparsedPublicKey::new(&signature::ECDSA_P256_SHA256_FIXED, buf.to_vec())
    }

    pub fn to_ring_bytes(&self) -> [u8; 65] {
        let mut buf = [0_u8; 65];
        buf[0] = 4;
        buf[1..33].copy_from_slice(&self.gx);
        buf[1..33].reverse();
        buf[33..].copy_from_slice(&self.gy);
        buf[33..].reverse();
        buf
    }
}

impl Secp256r1PrivateKey {
    #[cfg(feature = "openssl_support")]
    pub fn from_der(der_bytes: &[u8]) -> Secp256r1PrivateKey {
        let eckey = EcKey::private_key_from_der(der_bytes).unwrap();
        let mut prvkey_bytes_le = eckey.private_key().to_vec();
        prvkey_bytes_le.reverse();
        let bytes_len = prvkey_bytes_le.len();
        // for private keys with leading 0s, 0s will not be reflected in the vec
        // pad it with 0s
        let num_pad_bytes = 32 - bytes_len;
        if num_pad_bytes > 0 {
            prvkey_bytes_le.resize(bytes_len + num_pad_bytes, 0);
        }
        let mut seed = [0_u8; 32];
        seed.copy_from_slice(prvkey_bytes_le.as_slice());

        Secp256r1PrivateKey { r: seed }
    }

    #[cfg(not(feature = "substrate"))]
    pub fn from_sgx_ec256_private(key: &sgx_ec256_private_t) -> Secp256r1PrivateKey {
        Secp256r1PrivateKey { r: key.r }
    }

    #[cfg(not(feature = "substrate"))]
    pub fn to_sgx_ec256_private(&self) -> sgx_ec256_private_t {
        sgx_ec256_private_t { r: self.r }
    }

    pub fn to_raw_bytes(&self) -> [u8; 32] {
        let mut bytes = [0_u8; 32];
        bytes[..32].copy_from_slice(&self.r);
        bytes
    }
}

impl Secp256r1Signature {
    #[cfg(not(feature = "substrate"))]
    pub fn from_sgx_ec256_signature(sig: sgx_ec256_signature_t) -> Secp256r1Signature {
        Secp256r1Signature {
            x: unsafe { transmute::<[u32; 8], [u8; 32]>(sig.x) },
            y: unsafe { transmute::<[u32; 8], [u8; 32]>(sig.y) },
        }
    }

    #[cfg(not(feature = "substrate"))]
    pub fn to_sgx_ec256_signature(&self) -> sgx_ec256_signature_t {
        sgx_ec256_signature_t {
            x: unsafe { transmute::<[u8; 32], [u32; 8]>(self.x) },
            y: unsafe { transmute::<[u8; 32], [u32; 8]>(self.y) },
        }
    }

    #[cfg(feature = "ring_support")]
    pub fn from_ring_signature(ring_sig: &Signature) -> Secp256r1Signature {
        let ring_sig_buf = ring_sig.as_ref();
        assert_eq!(ring_sig_buf.len(), 64);

        let mut x: [u8; 32] = [0; 32];
        let mut y: [u8; 32] = [0; 32];
        x.copy_from_slice(&ring_sig_buf[..32]);
        y.copy_from_slice(&ring_sig_buf[32..]);
        x.reverse();
        y.reverse();
        Secp256r1Signature { x: x, y: y }
    }

    pub fn to_ring_signature_bytes(&self) -> [u8; 64] {
        let mut temp_buf: [u8; 64] = [0; 64];
        temp_buf[..32].copy_from_slice(&self.x);
        temp_buf[32..].copy_from_slice(&self.y);
        temp_buf[..32].reverse();
        temp_buf[32..].reverse();
        temp_buf
    }

    pub fn to_raw_bytes(&self) -> [u8; 64] {
        let mut bytes = [0_u8; 64];
        bytes[..32].copy_from_slice(&self.x);
        bytes[32..].copy_from_slice(&self.y);
        bytes
    }
}

#[cfg(not(feature = "substrate"))]
impl From<sgx_ec256_private_t> for Secp256r1PrivateKey {
    fn from(item: sgx_ec256_private_t) -> Self {
        Secp256r1PrivateKey::from_sgx_ec256_private(&item)
    }
}

#[cfg(not(feature = "substrate"))]
impl From<Secp256r1PrivateKey> for sgx_ec256_private_t {
    fn from(item: Secp256r1PrivateKey) -> Self {
        item.to_sgx_ec256_private()
    }
}

#[cfg(not(feature = "substrate"))]
impl From<sgx_ec256_public_t> for Secp256r1PublicKey {
    fn from(item: sgx_ec256_public_t) -> Self {
        Secp256r1PublicKey::from_sgx_ec256_public(&item)
    }
}

#[cfg(not(feature = "substrate"))]
impl From<Secp256r1PublicKey> for sgx_ec256_public_t {
    fn from(item: Secp256r1PublicKey) -> Self {
        item.to_sgx_ec256_public()
    }
}

#[cfg(not(feature = "substrate"))]
impl From<sgx_ec256_signature_t> for Secp256r1Signature {
    fn from(item: sgx_ec256_signature_t) -> Self {
        Secp256r1Signature::from_sgx_ec256_signature(item)
    }
}

#[cfg(not(feature = "substrate"))]
impl From<Secp256r1Signature> for sgx_ec256_signature_t {
    fn from(item: Secp256r1Signature) -> Self {
        item.to_sgx_ec256_signature()
    }
}

impl fmt::Display for Secp256r1PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Secp256r1PrivateKey").unwrap();
        writeln!(f, "r: {}", hex::encode(self.r))
    }
}

impl fmt::Display for Secp256r1PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Secp256r1PublicKey").unwrap();
        writeln!(f, "gx: {}", hex::encode(self.gx)).unwrap();
        writeln!(f, "gy: {}", hex::encode(self.gy))
    }
}

impl fmt::Display for Secp256r1Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Secp256r1Signature").unwrap();
        writeln!(f, "x: {}", hex::encode(self.x)).unwrap();
        writeln!(f, "y: {}", hex::encode(self.y))
    }
}
