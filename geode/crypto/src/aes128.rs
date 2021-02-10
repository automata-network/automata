// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "std")]
use sgx_ucrypto as intel_crypto;
#[cfg(feature = "std")]
use sgx_ucrypto::sgx_read_rand;

#[cfg(feature = "sgx_enclave")]
use sgx_tcrypto as intel_crypto;
#[cfg(feature = "sgx_enclave")]
use sgx_types::sgx_read_rand;

use geode_macros::handle_sgx;
use geode_types::*;
use intel_crypto::*;

use sgx_types::*;

use std::vec::Vec;

pub fn aes128cmac_mac(p_key: &Aes128Key, p_data: &[u8]) -> Result<Aes128Mac, CryptoError> {
    match rsgx_rijndael128_cmac_slice(&p_key.key, p_data) {
        Ok(mac) => Ok(Aes128Mac { mac: mac }),
        Err(s) => Err(CryptoError::SgxError(s.from_key(), format!("{}", s))),
    }
}

pub fn aes128cmac_verify(
    p_key: &Aes128Key,
    p_data: &[u8],
    p_orig_mac: &Aes128Mac,
) -> Result<bool, CryptoError> {
    let msg_mac = aes128cmac_mac(p_key, p_data)?;
    Ok(msg_mac.mac == p_orig_mac.mac)
}

pub fn aes128gcm_encrypt(
    p_key: &Aes128Key,
    p_data: &[u8],
) -> Result<Aes128EncryptedMsg, CryptoError> {
    let mut iv = [0_u8; 12];
    let mut mac = [0_u8; 16];
    let mut cipher = vec![0_u8; p_data.len()];

    unsafe {
        handle_sgx!(sgx_read_rand(iv.as_mut_ptr(), iv.len()))?;
    }
    if let Err(s) =
        rsgx_rijndael128GCM_encrypt(&p_key.key, p_data, &iv, &[], &mut cipher[..], &mut mac)
    {
        return Err(CryptoError::SgxError(s.from_key(), format!("{}", s)));
    };

    Ok(Aes128EncryptedMsg {
        iv: iv,
        mac: Aes128Mac { mac: mac },
        cipher: cipher,
    })
}

pub fn aes128gcm_decrypt(
    p_key: &Aes128Key,
    p_encrypted_msg: &Aes128EncryptedMsg,
) -> Result<Vec<u8>, CryptoError> {
    let p_iv = &p_encrypted_msg.iv;
    let p_mac = &p_encrypted_msg.mac.mac;
    let p_cipher = &p_encrypted_msg.cipher[..];
    let mut plaintext = vec![0_u8; p_encrypted_msg.cipher.len()];

    if let Err(s) =
        rsgx_rijndael128GCM_decrypt(&p_key.key, p_cipher, p_iv, &[], p_mac, &mut plaintext[..])
    {
        return Err(CryptoError::SgxError(s.from_key(), format!("{}", s)));
    }

    Ok(plaintext)
}
