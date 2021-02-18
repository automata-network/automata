// SPDX-License-Identifier: Apache-2.0

#![allow(unused_imports)]
#![allow(dead_code)]

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

use serde::de::Deserializer;
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};

#[cfg(feature = "sgx_enclave")]
use sgx_tstd::prelude::v1::*;

use ethereum_types::*;
use tiny_keccak::{Hasher, Keccak};

use crate::secp256k1::*;

pub fn keccak_hash(msg: &[u8]) -> [u8; 32] {
    // hash the message with keccak-256
    let mut keccak = Keccak::v256();
    let mut msg_hash = [0_u8; 32];
    keccak.update(msg);
    keccak.finalize(&mut msg_hash);
    msg_hash
}

pub fn parse_string_u256(u256_str: &str) -> U256 {
    if u256_str.starts_with("0x") {
        U256::from_str_radix(u256_str, 16).unwrap()
    } else {
        U256::from_str_radix(u256_str, 10).unwrap()
    }
}

pub fn parse_string_h160(h160_str: &str) -> H160 {
    let bytes = hex::decode(h160_str.trim_start_matches("0x")).unwrap();
    H160::from_slice(&bytes)
}

pub fn parse_string_h256(h256_str: &str) -> H256 {
    // hex string can be one of two forms
    // 1. 0x1123a5
    // 2.   1123a5
    // NOTE: for ethereum h256, the bytestring is represented in "big-endian" form
    // that is for an array of the form
    //   lsb [a5, 23, 11] msb
    // index: 0   1   2
    // the corresponding bytestring is of the form:
    // 0xa523110000..00
    //
    // Here, we'll strip the initial 0x and parse it using hex::decode
    // which gives us the exact representation we want.
    // 0xa5 23 11 00 .. 00
    //   a5 23 11 00 .. 00
    //  [a5,23,11,00,..,00] <- in the right endianness

    let bytes = hex::decode(h256_str.trim_start_matches("0x")).unwrap();
    // pad the bytes to 32bytes
    let mut padded_bytes = [0_u8; 32];
    padded_bytes[32 - bytes.len()..].copy_from_slice(&bytes);

    H256::from_slice(&padded_bytes)
}

pub fn encode_string_h256(h256: &H256) -> String {
    // here we just make use of the display functionality of H256
    // the debug string prints in full form (hex)
    format!("{:?}", h256)
}

pub fn eth_secp256k1_to_accountid(pubkey: &Secp256k1PublicKey) -> Address {
    let mut data = [0_u8; 64];
    data[..32].copy_from_slice(&pubkey.gx);
    data[32..].copy_from_slice(&pubkey.gy);

    let mut keccak = Keccak::v256();
    let mut msg_hash = [0_u8; 32];
    keccak.update(&data);
    keccak.finalize(&mut msg_hash);
    let mut addr_bytes = [0_u8; 20];
    addr_bytes.copy_from_slice(&msg_hash[12..]);
    addr_bytes.into()
}
