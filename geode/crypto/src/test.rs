// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn encrypt_decrypt() {
        let key = Aes128Key { key: [0; 16] };
        let data: [u8; 6] = [1, 2, 3, 4, 5, 6];
        let encrypted_msg = aes128gcm_encrypt(&key, &data).unwrap();
        let plaintext = aes128gcm_decrypt(&key, &encrypted_msg).unwrap();
        assert_eq!(plaintext, data);
    }

    #[test]
    fn mac_verify() {
        let key = Aes128Key { key: [0; 16] };
        let mut data: [u8; 6] = [1, 2, 3, 4, 5, 6];
        let data_mac = aes128cmac_mac(&key, &data).unwrap();
        assert_eq!(
            [194, 158, 14, 143, 248, 152, 4, 193, 94, 54, 74, 95, 115, 111, 30, 101],
            data_mac.mac
        );
        assert_eq!(true, aes128cmac_verify(&key, &data, &data_mac).unwrap());
        data[1] = 3;
        assert_eq!(false, aes128cmac_verify(&key, &data, &data_mac).unwrap());
    }

    #[test]
    fn derive_shared_secrets() {
        let (prvkey1, pubkey1) = secp256r1_gen_keypair().unwrap();
        let (prvkey2, pubkey2) = secp256r1_gen_keypair().unwrap();
        let kdk1 = derive_kdk(&prvkey1, &pubkey2).unwrap();
        let kdk2 = derive_kdk(&prvkey2, &pubkey1).unwrap();
        assert_eq!(kdk1, kdk2);
    }

    #[test]
    fn sign_verify() {
        let (prvkey1, pubkey1) = secp256r1_gen_keypair().unwrap();
        let (_prvkey2, pubkey2) = secp256r1_gen_keypair().unwrap();
        let msg = [1, 2, 3, 4, 5, 6];
        let mut signed_msg = secp256r1_sign_msg(&prvkey1, &msg).unwrap();
        assert_eq!(true, secp256r1_verify_msg(&pubkey1, &signed_msg).unwrap());
        assert_eq!(false, secp256r1_verify_msg(&pubkey2, &signed_msg).unwrap());
        signed_msg.msg[3] = 10;
        assert_eq!(false, secp256r1_verify_msg(&pubkey1, &signed_msg).unwrap());
        assert_eq!(false, secp256r1_verify_msg(&pubkey2, &signed_msg).unwrap());
    }

    #[test]
    fn secp256r1_from_der_test() {
        // priv:
        //     00:a6:5f:a6:65:d4:08:e5:4a:c2:61:9f:65:9e:b8:
        //     f0:d0:47:1c:b1:7c:bb:17:66:75:e9:65:56:43:df:
        //     af:ac
        // pub:
        //     04:b1:81:35:ac:6d:71:aa:ec:5a:79:33:73:85:e8:
        //     0c:c3:08:02:9e:15:9d:3e:9f:a5:53:53:bf:46:4e:
        //     ed:c0:84:5f:40:48:8e:f3:99:62:e3:42:79:2e:35:
        //     b4:24:48:e8:75:22:3e:7d:50:96:ee:a7:c8:42:c0:
        //     6d:d0:b7:69:17
        // ASN1 OID: prime256v1
        // NIST CURVE: P-256
        let aas_prvkey_der_bytes = [
            48, 119, 2, 1, 1, 4, 32, 0, 166, 95, 166, 101, 212, 8, 229, 74, 194, 97, 159, 101, 158,
            184, 240, 208, 71, 28, 177, 124, 187, 23, 102, 117, 233, 101, 86, 67, 223, 175, 172,
            160, 10, 6, 8, 42, 134, 72, 206, 61, 3, 1, 7, 161, 68, 3, 66, 0, 4, 177, 129, 53, 172,
            109, 113, 170, 236, 90, 121, 51, 115, 133, 232, 12, 195, 8, 2, 158, 21, 157, 62, 159,
            165, 83, 83, 191, 70, 78, 237, 192, 132, 95, 64, 72, 142, 243, 153, 98, 227, 66, 121,
            46, 53, 180, 36, 72, 232, 117, 34, 62, 125, 80, 150, 238, 167, 200, 66, 192, 109, 208,
            183, 105, 23,
        ];
        let prvkey = Secp256r1PrivateKey::from_der(&aas_prvkey_der_bytes);
        let prvkey_actual = Secp256r1PrivateKey {
            r: [
                172, 175, 223, 67, 86, 101, 233, 117, 102, 23, 187, 124, 177, 28, 71, 208, 240,
                184, 158, 101, 159, 97, 194, 74, 229, 8, 212, 101, 166, 95, 166, 0,
            ],
        };
        assert_eq!(prvkey, prvkey_actual);
    }

    #[test]
    fn sr25519_test() {
        let (sr25519_prvkey, sr25519_pubkey) = sr25519_gen_keypair().unwrap();
        let msg: &[u8] = b"test message";
        let mut signed_msg = sr25519_sign_msg(&sr25519_prvkey, msg).unwrap();
        assert_eq!(signed_msg.msg, msg);
        let is_verified = sr25519_verify_msg(&sr25519_pubkey, &signed_msg).unwrap();
        assert_eq!(true, is_verified);
        signed_msg.msg[0] = 0;
        let is_verified = sr25519_verify_msg(&sr25519_pubkey, &signed_msg).unwrap();
        assert_eq!(false, is_verified);
    }
}
