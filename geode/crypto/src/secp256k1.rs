// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "sgx_enclave")]
use sgx_types::sgx_read_rand;
#[cfg(feature = "std")]
use sgx_ucrypto::sgx_read_rand;

#[cfg(feature = "std")]
use serde;
#[cfg(feature = "std")]
use serde_json;

#[cfg(feature = "sgx_enclave")]
use serde_json_sgx as serde_json;
#[cfg(feature = "sgx_enclave")]
use serde_sgx as serde;

use serde::Serialize;

use geode_macros::handle_sgx;
use geode_types::*;

use sgx_types::*;

#[cfg(feature = "sgx_enclave")]
use sgx_tstd::prelude::v1::*;

// use rand::rngs::StdRng;
// use rand::SeedableRng;

use tiny_keccak::{Hasher, Keccak};

use rust_secp256k1::ffi::types::AlignedType;
use rust_secp256k1::{Message, PublicKey, Secp256k1, SecretKey};

pub fn secp256k1_gen_keypair() -> Result<(Secp256k1PrivateKey, Secp256k1PublicKey), CryptoError> {
    let mut seed_bytes = [0_u8; 32];
    unsafe {
        handle_sgx!(sgx_read_rand(seed_bytes.as_mut_ptr(), seed_bytes.len()))?;
    }

    let secp_size = Secp256k1::preallocate_size();
    let mut buf = vec![AlignedType::zeroed(); secp_size];
    let mut secp = Secp256k1::preallocated_new(&mut buf).unwrap();
    secp.seeded_randomize(&seed_bytes);

    let prvkey = loop {
        unsafe {
            handle_sgx!(sgx_read_rand(seed_bytes.as_mut_ptr(), seed_bytes.len()))?;
        }
        if let Ok(secret_key) = SecretKey::from_slice(&seed_bytes) {
            break secret_key;
        }
    };
    let pubkey = PublicKey::from_secret_key(&secp, &prvkey);

    Ok((prvkey.into(), pubkey.into()))
}

pub fn secp256k1_sign_msg<T: Serialize>(
    prvkey: &Secp256k1PrivateKey,
    msg: T,
) -> Result<Secp256k1SignedMsg<T>, CryptoError> {
    let msg_bytes = serde_json::to_vec(&msg).unwrap();
    let signature = secp256k1_sign_bytes(prvkey, &msg_bytes)?;
    Ok(Secp256k1SignedMsg { msg, signature })
}

pub fn secp256k1_sign_bytes(
    prvkey: &Secp256k1PrivateKey,
    msg: &[u8],
) -> Result<Secp256k1Signature, CryptoError> {
    let mut seed_bytes = [0_u8; 32];
    unsafe {
        handle_sgx!(sgx_read_rand(seed_bytes.as_mut_ptr(), seed_bytes.len()))?;
    }
    let secp_size = Secp256k1::preallocate_size();
    let mut buf = vec![AlignedType::zeroed(); secp_size];
    let mut secp = Secp256k1::preallocated_new(&mut buf).unwrap();
    secp.seeded_randomize(&seed_bytes);

    // hash the message with keccak-256
    let mut keccak = Keccak::v256();
    let mut msg_hash = [0_u8; 32];
    keccak.update(msg);
    keccak.finalize(&mut msg_hash);

    let message = Message::from_slice(&msg_hash).expect("32 bytes");
    let signature = secp.sign(&message, &(prvkey.clone().into()));

    Ok(signature.into())
}

pub fn secp256k1_public_from_private(
    prvkey: &Secp256k1PrivateKey,
) -> Result<Secp256k1PublicKey, CryptoError> {
    let mut seed_bytes = [0_u8; 32];
    unsafe {
        handle_sgx!(sgx_read_rand(seed_bytes.as_mut_ptr(), seed_bytes.len()))?;
    }
    let secp_size = Secp256k1::preallocate_size();
    let mut buf = vec![AlignedType::zeroed(); secp_size];
    let mut secp = Secp256k1::preallocated_new(&mut buf).unwrap();
    secp.seeded_randomize(&seed_bytes);
    let publickey = PublicKey::from_secret_key(&secp, &prvkey.clone().into());
    Ok(publickey.into())
}

pub fn secp256k1_rec_sign_bytes(
    prvkey: &Secp256k1PrivateKey,
    msg: &[u8],
) -> Result<Secp256k1RecoverableSignature, CryptoError> {
    let mut seed_bytes = [0_u8; 32];
    unsafe {
        handle_sgx!(sgx_read_rand(seed_bytes.as_mut_ptr(), seed_bytes.len()))?;
    }
    let secp_size = Secp256k1::preallocate_size();
    let mut buf = vec![AlignedType::zeroed(); secp_size];
    let mut secp = Secp256k1::preallocated_new(&mut buf).unwrap();
    secp.seeded_randomize(&seed_bytes);

    // hash the message with keccak-256
    let mut keccak = Keccak::v256();
    let mut msg_hash = [0_u8; 32];
    keccak.update(msg);
    keccak.finalize(&mut msg_hash);

    let message = Message::from_slice(&msg_hash).expect("32 bytes");
    let signature = secp.sign_recoverable(&message, &(prvkey.clone().into()));

    Ok(signature.into())
}

pub fn secp256k1_recover_pubkey(
    signature: &Secp256k1RecoverableSignature,
    msg: &[u8],
) -> Result<Secp256k1PublicKey, CryptoError> {
    let mut seed_bytes = [0_u8; 32];
    unsafe {
        handle_sgx!(sgx_read_rand(seed_bytes.as_mut_ptr(), seed_bytes.len()))?;
    }
    let secp_size = Secp256k1::preallocate_size();
    let mut buf = vec![AlignedType::zeroed(); secp_size];
    let mut secp = Secp256k1::preallocated_new(&mut buf).unwrap();
    secp.seeded_randomize(&seed_bytes);

    // hash the message with keccak-256
    let mut keccak = Keccak::v256();
    let mut msg_hash = [0_u8; 32];
    keccak.update(msg);
    keccak.finalize(&mut msg_hash);

    let message = Message::from_slice(&msg_hash).expect("32 bytes");
    Ok(secp
        .recover(&message, &signature.clone().into())
        .unwrap()
        .into())
}

pub fn secp256k1_eip712_sign<T: Serialize + EIP712>(
    prvkey: &Secp256k1PrivateKey,
    msg: EIP712Msg<T>,
) -> Result<EIP712SignedMsg<T>, CryptoError> {
    let data = msg.eip712_encode_msg();
    let signature = secp256k1_rec_sign_bytes(prvkey, &data)?;
    let result = EIP712SignedMsg {
        msg,
        v: signature.v + 27,
        r: signature.r.into(),
        s: signature.s.into(),
    };
    Ok(result)
}

pub fn secp256k1_eip712_verify<T: Serialize + EIP712>(
    signed_msg: &EIP712SignedMsg<T>,
) -> Result<Secp256k1PublicKey, CryptoError> {
    let rec_sig = signed_msg.into();
    let msg_bytes = signed_msg.msg.eip712_encode_msg();
    let pubkey = secp256k1_recover_pubkey(&rec_sig, &msg_bytes)?;
    Ok(pubkey)
}
