// SPDX-License-Identifier: Apache-2.0

#![allow(non_snake_case)]

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

use std::collections::BTreeMap;
use std::convert::TryFrom;

use crate::common::*;
use crate::eip712::*;
use crate::utils::*;
use ethereum_types::*;

big_array! { BigArray; }

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Vote {
    #[serde(deserialize_with = "deserialize_h256")]
    #[serde(serialize_with = "serialize_h256")]
    pub voter: H256,
    #[serde(deserialize_with = "deserialize_u256")]
    #[serde(serialize_with = "serialize_u256")]
    pub proposal: U256,
    pub option: u32,
    pub timestamp: u64,
}

impl EIP712 for Vote {
    fn type_hash() -> [u8; 32] {
        let members = Self::types()
            .iter()
            .map(|x| format!("{} {}", x.var_type, x.name))
            .collect::<Vec<String>>()
            .join(",");
        let encoded = format!("Vote({})", members);
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
        buf[..32].copy_from_slice(&self.voter[..]);
        self.proposal.to_big_endian(&mut buf[32..64]);
        Into::<U256>::into(self.option).to_big_endian(&mut buf[64..96]);
        Into::<U256>::into(self.timestamp).to_big_endian(&mut buf[96..128]);
        buf.to_vec()
    }

    fn types() -> Vec<EIP712VarType> {
        vec![
            EIP712VarType::new("voter", "uint256"),
            EIP712VarType::new("proposal", "uint256"),
            EIP712VarType::new("option", "uint32"),
            EIP712VarType::new("timestamp", "uint64"),
        ]
    }
}

impl Vote {
    pub fn new(voter: H256, proposal: U256, option: u32, timestamp: u64) -> Self {
        Vote {
            voter,
            proposal,
            option,
            timestamp,
        }
    }

    // although chainId is defined as U256, by ethereum definition, v is a single bytee,
    // v = v + chainid * 2 + 35
    // 256 > v + chainid * 2 + 35
    // v + chainid * 2 < 256 - 35
    // chainid * 2 < 256 - 35 - v
    // since v is either 0 or 1,
    // chainid < (256 - 35 - 1) / 2
    // chainid < 110
    // we can fit chainId in a single u8
    pub fn get_eip712_msg(&self, chainId: U256, verifyingContract: Address) -> EIP712Msg<Vote> {
        let types: BTreeMap<String, Vec<EIP712VarType>> = [
            ("EIP712Domain".to_owned(), EIP712Domain::types()),
            ("Vote".to_owned(), Vote::types()),
        ]
        .iter()
        .cloned()
        .collect();

        let domain = EIP712Domain::new("Witness", "0.1.0", chainId, verifyingContract);

        EIP712Msg::new(types, domain, "Vote", self.clone())
    }
}

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VoteReturnCode {
    Success,
    SignatureFailure,
    ProposalNotActive,
}

impl TryFrom<u32> for VoteReturnCode {
    type Error = ();
    fn try_from(n: u32) -> Result<Self, Self::Error> {
        match n {
            0 => Ok(VoteReturnCode::Success),
            1 => Ok(VoteReturnCode::SignatureFailure),
            2 => Ok(VoteReturnCode::ProposalNotActive),
            _ => Err(()),
        }
    }
}
