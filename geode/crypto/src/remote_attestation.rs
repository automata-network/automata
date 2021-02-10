// SPDX-License-Identifier: Apache-2.0

use crate::aes128::*;
use crate::secp256r1::*;
use geode_types::*;

pub fn aas_verify_reg_request(
    key: &Aes128Key,
    reg_request: &AasRegRequest,
) -> Result<bool, CryptoError> {
    let reg_request_bytes = reg_request.to_check_bytes();
    let reg_request_mac = aes128cmac_mac(key, &reg_request_bytes)?;
    Ok(reg_request_mac == reg_request.mac)
}

pub fn aas_verify_reg_report(
    pubkey: &Secp256r1PublicKey,
    reg_report: &AasRegReport,
) -> Result<bool, CryptoError> {
    let reg_report_bytes = reg_report.to_check_bytes();
    secp256r1_verify_signature(pubkey, &reg_report_bytes, &reg_report.aas_signature)
}

pub fn aas_sign_reg_report(
    prvkey: &Secp256r1PrivateKey,
    reg_report: AasRegReport,
) -> Result<AasRegReport, CryptoError> {
    let reg_report_bytes = reg_report.to_check_bytes();
    let signature = secp256r1_sign_bytes(prvkey, &reg_report_bytes)?;
    Ok(AasRegReport {
        attested_time: reg_report.attested_time,
        enclave_secp256r1_pubkey: reg_report.enclave_secp256r1_pubkey,
        enclave_sr25519_pubkey: reg_report.enclave_sr25519_pubkey,
        aas_signature: signature,
    })
}
