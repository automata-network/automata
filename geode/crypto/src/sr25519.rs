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

#[cfg(feature = "std")]
use sgx_ucrypto::sgx_read_rand;

#[cfg(feature = "sgx_enclave")]
use sgx_types::sgx_read_rand;

use sgx_types::*;

use rand::rngs::StdRng;
use rand::SeedableRng;
use schnorrkel;

use geode_macros::handle_sgx;
use geode_types::*;

const SIGNING_CONTEXT: &[u8] = b"substrate";

pub fn sr25519_gen_keypair() -> Result<(Sr25519PrivateKey, Sr25519PublicKey), CryptoError> {
    let mut seed_bytes = [0_u8; 32];
    unsafe {
        handle_sgx!(sgx_read_rand(seed_bytes.as_mut_ptr(), seed_bytes.len()))?;
    }
    let rng: StdRng = SeedableRng::from_seed(seed_bytes);
    let sr25519_keypair = schnorrkel::Keypair::generate_with(rng);
    let sr25519_pubkey = Sr25519PublicKey::from_schnorrkel_public(&sr25519_keypair.public);
    let sr25519_prvkey = Sr25519PrivateKey::from_schnorrkel_private(&sr25519_keypair.secret);
    Ok((sr25519_prvkey, sr25519_pubkey))
}

pub fn sr25519_sign_msg<T: Serialize>(
    prvkey: &Sr25519PrivateKey,
    msg: T,
) -> Result<Sr25519SignedMsg<T>, CryptoError> {
    let msg_bytes = serde_json::to_vec(&msg).unwrap();
    let signature = sr25519_sign_bytes(prvkey, &msg_bytes)?;
    Ok(Sr25519SignedMsg {
        msg: msg,
        signature: signature,
    })
}

pub fn sr25519_sign_bytes(
    prvkey: &Sr25519PrivateKey,
    msg: &[u8],
) -> Result<Sr25519Signature, CryptoError> {
    let mut seed_bytes = [0_u8; 32];
    unsafe {
        handle_sgx!(sgx_read_rand(seed_bytes.as_mut_ptr(), seed_bytes.len()))?;
    }
    let rng: StdRng = SeedableRng::from_seed(seed_bytes);
    let secretkey = prvkey.to_schnorrkel_private();
    let context = schnorrkel::signing_context(SIGNING_CONTEXT);
    let signature = secretkey.sign(
        schnorrkel::context::attach_rng(context.bytes(msg), rng),
        &secretkey.to_public(),
    );
    Ok(signature.into())
}

pub fn sr25519_verify_msg<T: Serialize>(
    pubkey: &Sr25519PublicKey,
    signed_msg: &Sr25519SignedMsg<T>,
) -> Result<bool, CryptoError> {
    let msg_bytes = serde_json::to_vec(&signed_msg.msg).unwrap();
    sr25519_verify_signature(pubkey, &msg_bytes, &signed_msg.signature)
}

pub fn sr25519_verify_signature(
    pubkey: &Sr25519PublicKey,
    msg: &[u8],
    signature: &Sr25519Signature,
) -> Result<bool, CryptoError> {
    let context = schnorrkel::signing_context(SIGNING_CONTEXT);
    let schnorrkel_pubkey = pubkey.to_schnorrkel_public();
    let schnorrkel_signature = signature.to_schnorrkel_signature();
    Ok(schnorrkel_pubkey
        .verify(context.bytes(msg), &schnorrkel_signature)
        .is_ok())
}

pub fn sr25519_ecdh_init(
    me: &Sr25519PrivateKey,
    you: &Sr25519PublicKey,
) -> Result<(Sr25519EcdhState, Sr25519SignedMsg<[u8; 32]>), CryptoError> {
    let mut seed_bytes = [0_u8; 32];
    unsafe {
        handle_sgx!(sgx_read_rand(seed_bytes.as_mut_ptr(), seed_bytes.len()))?;
    }
    let rng: StdRng = SeedableRng::from_seed(seed_bytes);
    let ephemeral_secret = x25519_dalek::EphemeralSecret::new(rng);
    let public_key = x25519_dalek::PublicKey::from(&ephemeral_secret);
    let signed_msg = sr25519_sign_msg(me, public_key.to_bytes()).unwrap();
    let state = Sr25519EcdhState {
        me: me.clone(),
        you: you.clone(),
        ephemeral_key: Some(ephemeral_secret),
    };
    Ok((state, signed_msg))
}

// consumes the state and a signed public key to produce a shared secret
pub fn sr25519_ecdh_compute(
    mut state: Sr25519EcdhState,
    signed_gb: Sr25519SignedMsg<[u8; 32]>,
) -> Result<[u8; 32], CryptoError> {
    if sr25519_verify_msg(&state.you, &signed_gb).unwrap() {
        let gb_bytes = signed_gb.msg;
        let gb = x25519_dalek::PublicKey::from(gb_bytes);
        let ephemeral_key = state.ephemeral_key.take().unwrap();
        let secret = ephemeral_key.diffie_hellman(&gb);
        Ok(secret.to_bytes())
    } else {
        Err(CryptoError::InvalidSignature)
    }
}
