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

#[cfg(feature = "ring_support")]
use ring::agreement;
#[cfg(feature = "ring_support")]
use ring::signature::{self, Signature};

#[cfg(feature = "substrate")]
use sp_std::prelude::*;

#[cfg(feature = "sgx_enclave")]
use sgx_tstd::prelude::v1::*;

big_array! { BigArray; }

use crate::aes128::*;
use crate::secp256r1::*;
use crate::sr25519::*;

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(
    Serialize, Deserialize, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone,
)]
pub struct AasRegRequest {
    pub enclave_secp256r1_pubkey: Secp256r1PublicKey,
    pub enclave_sr25519_pubkey: Sr25519PublicKey,
    pub mac: Aes128Mac,
}

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(
    Serialize, Deserialize, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone,
)]
pub struct AasRegReport {
    pub attested_time: u64,
    pub enclave_secp256r1_pubkey: Secp256r1PublicKey,
    pub enclave_sr25519_pubkey: Sr25519PublicKey,
    pub aas_signature: Secp256r1Signature,
}

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
pub struct AliveEvidence {
    pub magic_str: [u8; 8],
    pub task_id: Vec<u8>,
    pub block_hash: Vec<u8>,
    pub data_in: usize,
    pub data_out: usize,
    pub storage_in: usize,
    pub storage_out: usize,
    pub storage_size: usize,
    pub compute: usize,
}

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
pub struct AasTimestamp {
    pub timestamp: u64,
    pub data: Vec<u8>,
}

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct SgxRaMsg3Reply {
    pub is_verified: bool,
    pub tcb_update: Option<Vec<u8>>,
}

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct SgxRaFinalizeRequest {
    pub signed_claiminfo: Sr25519SignedMsg<ProviderClaimInfo>,
}

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy)]
pub struct ProviderClaimInfo {
    pub geode_pubkey: Sr25519PublicKey,
    pub provider_pubkey: Sr25519PublicKey,
}

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct AttestorInfo {
    pub uri: String,
    pub secp256r1_pubkey: Secp256r1PublicKey,
}

impl AasRegRequest {
    // enclave_secp256r1_pubkey - 64
    // enclave_sr25519pubkey    - 32
    // mac                     - 16
    pub fn to_raw_bytes(&self) -> [u8; 112] {
        let mut bytes = [0_u8; 112];
        bytes[..64].copy_from_slice(&self.enclave_secp256r1_pubkey.to_raw_bytes());
        bytes[64..96].copy_from_slice(&self.enclave_sr25519_pubkey.to_raw_bytes());
        bytes[96..].copy_from_slice(&self.mac.to_raw_bytes());
        bytes
    }

    pub fn to_check_bytes(&self) -> [u8; 96] {
        let mut bytes = [0_u8; 96];
        bytes.copy_from_slice(&self.to_raw_bytes()[..96]);
        bytes
    }
}

impl AasRegReport {
    // attested_time             - 8
    // enclave_secp256r1_pubkey   - 64
    // enclave_sr25519_pubkey     - 32
    // aas_signature             - 64
    pub fn to_raw_bytes(&self) -> [u8; 168] {
        let mut bytes = [0_u8; 168];
        bytes[..8].copy_from_slice(&self.attested_time.to_le_bytes());
        bytes[8..72].copy_from_slice(&self.enclave_secp256r1_pubkey.to_raw_bytes());
        bytes[72..104].copy_from_slice(&self.enclave_sr25519_pubkey.to_raw_bytes());
        bytes[104..].copy_from_slice(&self.aas_signature.to_raw_bytes());
        bytes
    }

    pub fn to_check_bytes(&self) -> [u8; 104] {
        let mut bytes = [0_u8; 104];
        bytes.copy_from_slice(&self.to_raw_bytes()[..104]);
        bytes
    }
}
