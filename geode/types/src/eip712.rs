// SPDX-License-Identifier: Apache-2.0

#![allow(non_snake_case)]
#![allow(unused_imports)]

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

#[cfg(feature = "substrate")]
use sp_std::prelude::*;

#[cfg(feature = "sgx_enclave")]
use sgx_tstd::prelude::v1::*;

use ethereum_types::*;

use crate::common::*;
use crate::utils::*;

//use std::convert::TryFrom;
use std::collections::BTreeMap;

pub trait EIP712 {
    fn type_hash() -> [u8; 32];
    fn encode_data(&self) -> Vec<u8>;
    fn hash_struct(&self) -> [u8; 32];
    fn types() -> Vec<EIP712VarType>;
}

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct EIP712VarType {
    pub name: String,
    #[serde(rename = "type")]
    pub var_type: String,
}

impl EIP712VarType {
    pub fn new(name: &str, var_type: &str) -> Self {
        EIP712VarType {
            name: name.to_owned(),
            var_type: var_type.to_owned(),
        }
    }
}

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(Deserialize, Default, Debug, Clone)]
pub struct EIP712Domain {
    pub name: String,
    pub version: String,
    #[serde(deserialize_with = "deserialize_u256")]
    pub chainId: U256,
    #[serde(deserialize_with = "deserialize_address")]
    pub verifyingContract: Address,
}

impl EIP712Domain {
    pub fn new(name: &str, version: &str, chainId: U256, contract: Address) -> Self {
        EIP712Domain {
            name: name.to_owned(),
            version: version.to_owned(),
            chainId: chainId,
            verifyingContract: contract,
        }
    }
}

impl EIP712 for EIP712Domain {
    fn type_hash() -> [u8; 32] {
        let members = Self::types()
            .iter()
            .map(|x| format!("{} {}", x.var_type, x.name))
            .collect::<Vec<String>>()
            .join(",");
        let encoded = format!("EIP712Domain({})", members);
        keccak_hash(encoded.as_bytes())
    }

    fn hash_struct(&self) -> [u8; 32] {
        let mut data: Vec<u8> = Vec::new();
        data.extend(&Self::type_hash());
        data.extend(&self.encode_data());
        keccak_hash(&data)
    }

    fn encode_data(&self) -> Vec<u8> {
        let mut buf = [0_u8; 32 * 4];
        buf[..32].copy_from_slice(&keccak_hash(self.name.as_bytes()));
        buf[32..64].copy_from_slice(&keccak_hash(self.version.as_bytes()));
        self.chainId.to_big_endian(&mut buf[64..96]);
        buf[108..128].copy_from_slice(self.verifyingContract.as_bytes());
        buf.to_vec()
    }

    fn types() -> Vec<EIP712VarType> {
        vec![
            EIP712VarType::new("name", "string"),
            EIP712VarType::new("version", "string"),
            EIP712VarType::new("chainId", "uint256"),
            EIP712VarType::new("verifyingContract", "address"),
        ]
    }
}

impl Serialize for EIP712Domain {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("EIP712Domain", 4)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("version", &self.version)?;
        state.serialize_field("chainId", &format!("{:?}", &self.chainId))?;
        state.serialize_field(
            "verifyingContract",
            &format!("{:?}", &self.verifyingContract),
        )?;
        state.end()
    }
}

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct EIP712Msg<T>
where
    T: EIP712,
{
    pub types: BTreeMap<String, Vec<EIP712VarType>>,
    pub domain: EIP712Domain,
    pub primaryType: String,
    pub message: T,
}

impl<T> EIP712Msg<T>
where
    T: EIP712,
{
    pub fn new(
        types: BTreeMap<String, Vec<EIP712VarType>>,
        domain: EIP712Domain,
        primaryType: &str,
        message: T,
    ) -> Self {
        EIP712Msg {
            types: types,
            domain: domain,
            primaryType: primaryType.to_owned(),
            message: message,
        }
    }

    pub fn eip712_encode_msg(&self) -> [u8; 66] {
        // encode(domainSeparator : 𝔹²⁵⁶, message : 𝕊) = "\x19\x01" ‖ domainSeparator ‖
        // hashStruct(message) where domainSeparator and hashStruct(message) are defined below.
        //
        // domainSeparator = hashStruct(eip712Domain)

        let mut data = [0_u8; 66];
        data[0] = 0x19;
        data[1] = 0x01;
        data[2..34].copy_from_slice(&self.domain.hash_struct());
        data[34..66].copy_from_slice(&self.message.hash_struct());
        data
    }
}

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(Deserialize, Default, Debug, Clone)]
pub struct EIP712SignedMsg<T>
where
    T: EIP712,
{
    pub msg: EIP712Msg<T>,
    pub v: u8,
    #[serde(deserialize_with = "deserialize_h256")]
    pub r: H256,
    #[serde(deserialize_with = "deserialize_h256")]
    pub s: H256,
}

impl<'de, T: Serialize + Deserialize<'de>> Serialize for EIP712SignedMsg<T>
where
    T: EIP712,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("EIP712SignedMsg", 4)?;
        state.serialize_field("msg", &self.msg)?;
        state.serialize_field("v", &self.v)?;
        state.serialize_field("r", &format!("{:?}", &self.r))?;
        state.serialize_field("s", &format!("{:?}", &self.s))?;
        state.end()
    }
}
