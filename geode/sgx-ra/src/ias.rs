// SPDX-License-Identifier: Apache-2.0

use std::mem::transmute;

use base16;
use base64;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use std::str::FromStr;
use strum_macros::{Display, EnumString};

use std::error::Error;

use crate::sgx_ra::SgxQuote;
use geode_types::*;

use bitflags::bitflags;
use sgx_types::*;

#[derive(Default, Clone)]
pub struct IasServer {
    api_key: String,
    base_url: &'static str,
}

impl IasServer {
    pub fn new(apikey: &str, is_dev: bool) -> IasServer {
        let base_url = if is_dev {
            "https://api.trustedservices.intel.com/sgx/dev"
        } else {
            "https://api.trustedservices.intel.com/sgx"
        };
        IasServer {
            api_key: String::from(apikey),
            base_url: base_url,
        }
    }

    pub fn verify_quote(&self, quote: SgxQuote) -> Result<IasReportResponse, Box<dyn Error>> {
        let quote_bytes = quote.as_bytes();

        // get a random nonce
        let random_nonce: String = thread_rng().sample_iter(&Alphanumeric).take(32).collect();

        let report_request = IasReportRequest {
            isv_enclave_quote: base64::encode(quote_bytes),
            nonce: Some(random_nonce),
        };
        let report_request_json = serde_json::to_string(&report_request).unwrap();

        let api_url = format!("{}/attestation/v4/report", self.base_url);
        let client = reqwest::blocking::Client::new();
        let res = client
            .post(&api_url)
            .header("Content-Type", "application/json")
            .header("Ocp-Apim-Subscription-Key", self.api_key.as_str())
            .body(report_request_json)
            .send()?;
        let avr_json = res.text()?;
        let avr: IasReportResponse = serde_json::from_str(&avr_json)?;
        Ok(avr)
    }

    pub fn get_sigrl(&self, gid: &[u8; 4]) -> Vec<u8> {
        let mut gid_be = [0_u8; 4];
        gid_be.copy_from_slice(gid);
        gid_be.reverse();
        let gid_base16 = base16::encode_lower(&gid_be);
        let api_url = format!("{}/attestation/v4/sigrl/{}", self.base_url, gid_base16);
        let client = reqwest::blocking::Client::new();
        let res = client
            .get(&api_url)
            .header("Ocp-Apim-Subscription-Key", self.api_key.as_str())
            .send()
            .unwrap();
        let sigrl_base64 = res.text().unwrap();
        base64::decode(sigrl_base64).unwrap()
    }
}

bitflags! {
    /* Masks for sgx_epid_group_flags*/
    #[derive(Default, Serialize, Deserialize)]
    pub struct SgxEpidGroupFlags: u8 {
        const QE_EPID_GROUP_REVOKED                  = 0b00000001;
        const PERF_REKEY_FOR_QE_EPID_GROUP_AVAILABLE = 0b00000010;
        const QE_EPID_GROUP_OUT_OF_DATE              = 0b00000100;
    }
}

bitflags! {
    /* Masks for sgx_tcb_evaluation_flags*/
    #[derive(Default, Serialize, Deserialize)]
    pub struct SgxTcbEvaluationFlags: u16 {
        const QUOTE_CPUSVN_OUT_OF_DATE      = 0b00000001;
        const QUOTE_ISVSVN_QE_OUT_OF_DATE   = 0b00000010;
        const QUOTE_ISVSVN_PCE_OUT_OF_DATE  = 0b00000100;
        /* the EPID signature of the ISV enclave QUOTE has been verified correctly but additional
         * configuration of SGX platform may be needed.*/
        const PLATFORM_CONFIGURATION_NEEDED = 0b00001000;
    }
}

bitflags! {
    /* Masks for sgx_pse_evaluation_flags*/
    #[derive(Default, Serialize, Deserialize)]
    pub struct PseEvaluationFlags: u16 {
        /* PS_SEC_PROP_DESC.PSE_ISVSVN is out of date*/
        const PSE_ISVSVN_OUT_OF_DATE                          = 0b00000001;
        /* CSME EPID 1.1 group identified by PS_SEC_PROP_DESC. PS_HW_GID has been revoked*/
        const EPID_GROUP_ID_BY_PS_HW_GID_REVOKED              = 0b00000010;
        /* PSDA SVN indicated in PS_SEC_PROP_DESC.PS_HW_SEC_INFO is out of date*/
        const SVN_FROM_PS_HW_SEC_INFO_OUT_OF_DATE             = 0b00000100;
        /* CSME EPID 1.1 SigRL version indicated in PS_SEC_PROP_DESC. PS_HW_SIG_RLver is out of
         * date*/
        const SIGRL_VER_FROM_PS_HW_SIG_RLVER_OUT_OF_DATE      = 0b00001000;
        /* CSME EPID 1.1 PrivRL version indicated in PS_SEC_PROP_DESC. PS_HW_PrivKey_RLver is out
         * of date*/
        const PRIVRL_VER_FROM_PS_HW_PRV_KEY_RLVER_OUT_OF_DATE = 0b00010000;
    }
}

const SGX_CPUSVN_SIZE: usize = 16;
const PSVN_SIZE: usize = 18; // sizeof(psvn_t)
const PSDA_SVN_SIZE: usize = 4;
const ISVSVN_SIZE: usize = 2;
const SGX_PLATFORM_INFO_SIZE: usize = 101;

#[allow(non_camel_case_types)]
type sgx_isv_svn_t = u16; // 2 bytes
#[allow(non_camel_case_types)]
type tcb_psvn_t = [u8; PSVN_SIZE];
#[allow(non_camel_case_types)]
type psda_svn_t = [u8; PSDA_SVN_SIZE];
#[allow(non_camel_case_types)]
type pse_isvsvn_t = [u8; ISVSVN_SIZE];

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
#[allow(non_camel_case_types)]
struct sgx_cpu_svn_t {
    // 16 bytes
    svn: [u8; SGX_CPUSVN_SIZE],
}

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
#[allow(non_camel_case_types)]
struct psvn_t {
    // 16 + 2
    cpu_svn: sgx_cpu_svn_t,
    isv_svn: sgx_isv_svn_t,
}

#[derive(Copy, Clone)]
#[repr(packed)]
#[allow(non_camel_case_types)]
struct platform_info_blob {
    sgx_epid_group_flags: u8,
    sgx_tcb_evaluation_flags: u16,
    pse_evaluation_flags: u16,
    latest_equivalent_tcb_psvn: tcb_psvn_t,
    latest_pse_isvsvn: pse_isvsvn_t,
    latest_psda_svn: psda_svn_t,
    xeid: u32,
    gid: u32,
    signature: sgx_ec256_signature_t,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IasPlatformInfoBlob {
    sgx_epid_group_flags: SgxEpidGroupFlags,
    sgx_tcb_evaluation_flags: SgxTcbEvaluationFlags,
    pse_evaluation_flags: PseEvaluationFlags,
    latest_equivalent_tcb_psvn: [u8; PSVN_SIZE],
    latest_pse_isvsvn: [u8; ISVSVN_SIZE],
    latest_psda_svn: [u8; PSDA_SVN_SIZE],
    xeid: u32,
    gid: u32,
    signature: Secp256r1Signature,
}

impl From<[u8; SGX_PLATFORM_INFO_SIZE]> for IasPlatformInfoBlob {
    fn from(blob_bytes: [u8; SGX_PLATFORM_INFO_SIZE]) -> Self {
        let parsed_blob: platform_info_blob = unsafe { transmute(blob_bytes) };
        IasPlatformInfoBlob {
            sgx_epid_group_flags: SgxEpidGroupFlags::from_bits(parsed_blob.sgx_epid_group_flags)
                .unwrap(),
            // https://github.com/intel/linux-sgx/blob/33f4499173497bdfdf72c5f61374c0fadc5c5365/psw/ae/aesm_service/source/bundles/epid_quote_service_bundle/platform_info_facility.cpp#L59
            // const uint16_t* p = reinterpret_cast<const uint16_t*>(p_platform_info_blob->platform_info_blob.sgx_tcb_evaluation_flags);
            // *pflags = lv_ntohs(*p);
            sgx_tcb_evaluation_flags: SgxTcbEvaluationFlags::from_bits(u16::from_be(
                parsed_blob.sgx_tcb_evaluation_flags,
            ))
            .unwrap(),
            pse_evaluation_flags: PseEvaluationFlags::from_bits(u16::from_be(
                parsed_blob.pse_evaluation_flags,
            ))
            .unwrap(),
            latest_equivalent_tcb_psvn: parsed_blob.latest_equivalent_tcb_psvn,
            latest_pse_isvsvn: parsed_blob.latest_pse_isvsvn,
            latest_psda_svn: parsed_blob.latest_psda_svn,
            xeid: parsed_blob.xeid,
            gid: parsed_blob.gid,
            signature: parsed_blob.signature.into(),
        }
    }
}

impl From<[u8; SGX_PLATFORM_INFO_SIZE + 4]> for IasPlatformInfoBlob {
    fn from(blob_bytes: [u8; SGX_PLATFORM_INFO_SIZE + 4]) -> Self {
        // Remove the TSV header (undocumented)
        let pib_vec = blob_bytes[4..].to_vec();
        let mut pib_array: [u8; 101] = [0; 101];
        pib_array.copy_from_slice(&pib_vec);
        pib_array.into()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IasReportRequest {
    pub isv_enclave_quote: String,
    pub nonce: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IasReportResponse {
    pub id: String,
    pub timestamp: String,
    pub version: u32,
    pub isv_enclave_quote_status: String,
    pub isv_enclave_quote_body: String,
    pub revocation_reason: Option<String>,
    pub pse_manifest_status: Option<String>,
    pub pse_manifest_hash: Option<String>,
    pub platform_info_blob: Option<String>,
    pub nonce: Option<String>,
    pub epid_pseudonym: Option<String>,
    #[serde(rename(serialize = "advisoryURL"))]
    #[serde(rename(deserialize = "advisoryURL"))]
    pub advisory_url: Option<String>,
    #[serde(rename(serialize = "advisoryIDs"))]
    #[serde(rename(deserialize = "advisoryIDs"))]
    pub advisory_ids: Option<Vec<String>>,
}

impl IasReportResponse {
    pub fn get_isv_enclave_quote_body(&self) -> SgxQuote {
        let isv_enclave_quote_body = base64::decode(&self.isv_enclave_quote_body).unwrap();
        // size of sgx_quote_t is 436 bytes,
        // isv_enclave_quote_body don't have signature and signature len
        SgxQuote::from_isv_bytes(isv_enclave_quote_body).unwrap()
    }

    pub fn get_isv_enclave_quote_status(&self) -> String {
        self.isv_enclave_quote_status.to_owned()
    }

    pub fn is_enclave_secure(&self, allow_conditional: bool) -> bool {
        use EnclaveQuoteStatus::*;

        let isv_enclave_quote_status =
            EnclaveQuoteStatus::from_str(&self.isv_enclave_quote_status).unwrap();
        let is_secure = match isv_enclave_quote_status {
            Ok => true,
            SignatureInvalid => false,
            GroupRevoked => false,
            SignatureRevoked => false,
            KeyRevoked => false,
            SigrlVersionMismatch => false,
            // the following items are conditionally "secure"
            GroupOutOfDate => allow_conditional,
            ConfigurationNeeded => allow_conditional,
            SwHardeningNeeded => allow_conditional,
            ConfigurationAndSwHardeningNeeded => allow_conditional,
        };
        is_secure
    }
}

#[derive(Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
enum EnclaveQuoteStatus {
    Ok,
    SignatureInvalid,
    GroupRevoked,
    SignatureRevoked,
    KeyRevoked,
    SigrlVersionMismatch,
    GroupOutOfDate,
    ConfigurationNeeded,
    SwHardeningNeeded,
    ConfigurationAndSwHardeningNeeded,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
