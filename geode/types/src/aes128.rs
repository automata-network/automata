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

#[cfg(feature = "substrate")]
use sp_std::prelude::*;

#[cfg(feature = "sgx_enclave")]
use sgx_tstd::prelude::v1::*;

big_array! { BigArray; }

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(
    Serialize, Deserialize, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone,
)]
pub struct Aes128Key {
    pub key: [u8; 16],
}

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(
    Serialize, Deserialize, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone,
)]
pub struct Aes128Mac {
    pub mac: [u8; 16],
}

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
pub struct Aes128EncryptedMsg {
    pub iv: [u8; 12],
    pub mac: Aes128Mac,
    pub cipher: Vec<u8>,
}

impl Aes128Key {
    pub fn from_slice(byte_slice: &[u8]) -> Aes128Key {
        let mut buf = [0_u8; 16];
        buf.copy_from_slice(byte_slice);
        Aes128Key { key: buf }
    }

    pub fn to_raw_bytes(&self) -> [u8; 16] {
        self.key
    }
}

impl Aes128Mac {
    pub fn to_raw_bytes(&self) -> [u8; 16] {
        self.mac
    }
}

#[cfg(not(feature = "substrate"))]
impl From<[u8; 16]> for Aes128Key {
    fn from(item: [u8; 16]) -> Self {
        Aes128Key { key: item }
    }
}
