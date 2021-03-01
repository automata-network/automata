// SPDX-License-Identifier: Apache-2.0

use sgx_types::*;

use geode_macros::handle_sgx;
use geode_types::*;

#[allow(dead_code)]
pub fn enclave_get_sk_key(ra_context: sgx_ra_context_t) -> Result<Aes128Key, CryptoError> {
    let mut key = sgx_key_128bit_t::default();
    unsafe {
        handle_sgx!(sgx_ra_get_keys(
            ra_context,
            sgx_ra_key_type_t::SGX_RA_KEY_SK,
            &mut key
        ))?;
    };
    Ok(Aes128Key { key })
}
