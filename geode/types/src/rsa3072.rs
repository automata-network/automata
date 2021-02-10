// SPDX-License-Identifier: Apache-2.0

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(Serialize, Deserialize)]
pub struct Rsa3072Signature {
    #[serde(with = "BigArray")]
    signature: [u8; 384],
}

#[cfg_attr(feature = "sgx_enclave", serde(crate = "serde_sgx"))]
#[cfg_attr(feature = "substrate", serde(crate = "serde_substrate"))]
#[derive(Serialize, Deserialize)]
pub struct Rsa3072PublicKey {
    #[serde(with = "BigArray")]
    modulus: [u8; 384],
    exponent: [u8; 4],
}



impl Default for Rsa3072Signature {
    fn default() -> Self {
        Rsa3072Signature {
            signature: [0; 384],
        }
    }
}

impl Default for Rsa3072PublicKey {
    fn default() -> Self {
        Rsa3072PublicKey {
            modulus: [0; 384],
            exponent: [0; 4],
        }
    }
}

// #[derive(Serialize, Deserialize)]
// #[serde(remote = "sgx_rsa3072_signature_t")]
// struct _SgxRsa3072Signature {
//     #[serde(with = "BigArray")]
//     signature: [u8; 384],
// }
//
// #[derive(Serialize, Deserialize)]
// #[serde(remote = "sgx_rsa3072_public_key_t")]
// struct _SgxRsa3072PublicKey {
//     #[serde(with = "BigArray")]
//     modulus: [u8; 384],
//     exponent: [u8; 4],
// }
