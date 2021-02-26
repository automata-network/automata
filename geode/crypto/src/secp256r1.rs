// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "std")]
use serde;
#[cfg(feature = "std")]
use serde_json;

#[cfg(feature = "sgx_enclave")]
use serde_json_sgx as serde_json;
#[cfg(feature = "sgx_enclave")]
use serde_sgx as serde;

use serde::Serialize;

use crate::aes128::*;
use geode_macros::handle_sgx;
use geode_types::*;
use sgx_types::*;

pub fn secp256r1_gen_keypair() -> Result<(Secp256r1PrivateKey, Secp256r1PublicKey), CryptoError> {
    // generate secp256r1 keypair for communication with worker
    let mut sgx_pubkey = sgx_ec256_public_t::default();
    let mut sgx_prvkey = sgx_ec256_private_t::default();
    let mut ecc_handle: sgx_ecc_state_handle_t = 0 as sgx_ecc_state_handle_t;

    unsafe {
        handle_sgx!(sgx_ecc256_open_context(&mut ecc_handle))?;
        handle_sgx!(sgx_ecc256_create_key_pair(
            &mut sgx_prvkey,
            &mut sgx_pubkey,
            ecc_handle
        ))?;
        handle_sgx!(sgx_ecc256_close_context(ecc_handle))?;
    }
    let prvkey = Secp256r1PrivateKey::from_sgx_ec256_private(&sgx_prvkey);
    let pubkey = Secp256r1PublicKey::from_sgx_ec256_public(&sgx_pubkey);
    Ok((prvkey, pubkey))
}

pub fn secp256r1_compute_shared_dhkey(
    prvkey: &Secp256r1PrivateKey,
    pubkey: &Secp256r1PublicKey,
) -> Result<[u8; 32], CryptoError> {
    let mut sgx_prvkey = prvkey.to_sgx_ec256_private();
    let mut sgx_pubkey = pubkey.to_sgx_ec256_public();
    let mut gab_x = sgx_ec256_dh_shared_t::default();
    let mut ecc_handle: sgx_ecc_state_handle_t = 0 as sgx_ecc_state_handle_t;

    unsafe {
        handle_sgx!(sgx_ecc256_open_context(&mut ecc_handle))?;
        handle_sgx!(sgx_ecc256_compute_shared_dhkey(
            &mut sgx_prvkey,
            &mut sgx_pubkey,
            &mut gab_x,
            ecc_handle
        ))?;
        handle_sgx!(sgx_ecc256_close_context(ecc_handle))?;
    }
    Ok(gab_x.s)
}

pub fn derive_kdk(
    prvkey: &Secp256r1PrivateKey,
    pubkey: &Secp256r1PublicKey,
) -> Result<Aes128Key, CryptoError> {
    let shared_dhkey = secp256r1_compute_shared_dhkey(prvkey, pubkey)?;
    let key0 = Aes128Key { key: [0; 16] };

    let mac = aes128cmac_mac(&key0, &shared_dhkey)?;
    Ok(Aes128Key { key: mac.mac })
}

pub fn secp256r1_sign_msg<T: Serialize>(
    prvkey: &Secp256r1PrivateKey,
    msg: T,
) -> Result<Secp256r1SignedMsg<T>, CryptoError> {
    let msg_bytes = serde_json::to_vec(&msg).unwrap();
    let signature = secp256r1_sign_bytes(prvkey, &msg_bytes)?;

    Ok(Secp256r1SignedMsg::<T> {
        msg,
        signature,
    })
}

pub fn secp256r1_sign_bytes(
    prvkey: &Secp256r1PrivateKey,
    msg: &[u8],
) -> Result<Secp256r1Signature, CryptoError> {
    let mut sgx_prvkey = prvkey.to_sgx_ec256_private();
    let mut ecc_handle: sgx_ecc_state_handle_t = 0 as sgx_ecc_state_handle_t;
    let mut signature = sgx_ec256_signature_t::default();

    unsafe {
        handle_sgx!(sgx_ecc256_open_context(&mut ecc_handle))?;
        handle_sgx!(sgx_ecdsa_sign(
            msg.as_ptr(),
            msg.len() as u32,
            &mut sgx_prvkey,
            &mut signature,
            ecc_handle
        ))?;
        handle_sgx!(sgx_ecc256_close_context(ecc_handle))?;
    }

    Ok(signature.into())
}

pub fn secp256r1_verify_msg<T: Serialize>(
    pubkey: &Secp256r1PublicKey,
    signed_msg: &Secp256r1SignedMsg<T>,
) -> Result<bool, CryptoError> {
    let msg_bytes = serde_json::to_vec(&signed_msg.msg).unwrap();
    secp256r1_verify_signature(pubkey, &msg_bytes, &signed_msg.signature)
}

pub fn secp256r1_verify_signature(
    pubkey: &Secp256r1PublicKey,
    msg: &[u8],
    signature: &Secp256r1Signature,
) -> Result<bool, CryptoError> {
    let sgx_pubkey = pubkey.to_sgx_ec256_public();
    let mut sgx_signature = signature.to_sgx_ec256_signature();
    let mut ecc_handle: sgx_ecc_state_handle_t = 0 as sgx_ecc_state_handle_t;
    let mut result = 0;

    unsafe {
        handle_sgx!(sgx_ecc256_open_context(&mut ecc_handle))?;
        handle_sgx!(sgx_ecdsa_verify(
            msg.as_ptr(),
            msg.len() as u32,
            &sgx_pubkey,
            &mut sgx_signature,
            &mut result,
            ecc_handle
        ))?;
        handle_sgx!(sgx_ecc256_close_context(ecc_handle))?;
    }
    let result = sgx_generic_ecresult_t::from_repr(result as u32).unwrap();
    match result {
        sgx_generic_ecresult_t::SGX_EC_VALID => Ok(true),
        sgx_generic_ecresult_t::SGX_EC_INVALID_SIGNATURE => Ok(false),
        e => panic!(
            "Unexpected ECC Result! {:?} (Refer to sgx_generic_ecresult_t)",
            e.from_key()
        ),
    }
}
