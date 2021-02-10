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

use ethereum_types::*;
use tiny_keccak::{Hasher, Keccak};

use crate::secp256k1::*;
use crate::utils::*;

#[cfg(feature = "sgx_enclave")]
use sgx_tstd::prelude::v1::*;

pub fn deserialize_u256<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(parse_string_u256(&s))
}

pub fn deserialize_address<'de, D>(deserializer: D) -> Result<Address, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let bytes = hex::decode(s.trim_start_matches("0x")).unwrap();
    Ok(Address::from_slice(&bytes))
}

pub fn serialize_u256<S>(item: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let item_str = format!("{}", item);
    serializer.serialize_str(&item_str)
}

pub fn serialize_h256<S>(item: &H256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&encode_string_h256(item))
}

pub fn deserialize_h256<'de, D>(deserializer: D) -> Result<H256, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(parse_string_h256(&s))
}
