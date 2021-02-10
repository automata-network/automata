// SPDX-License-Identifier: Apache-2.0

use cargo_toml::Manifest;
use lazy_static::lazy_static;
use openssl::bn::BigNum;
use openssl::rsa::Rsa;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

mod enclave;
mod geode;
mod intel;
mod teaclave;

pub use enclave::Build as EnclaveBuild;
pub use geode::Build as GeodeBuild;

lazy_static! {
    static ref CARGO_MANIFEST_DIR: String = env::var("CARGO_MANIFEST_DIR").unwrap();
    static ref PROFILE: String = env::var("PROFILE").unwrap();
    static ref OUT_DIR: String = env::var("OUT_DIR").unwrap();
    static ref ENCLAVE_SIGNING_KEY: String = {
        if let Ok(val) = env::var("ENCLAVE_SIGNING_KEY") {
            val
        } else {
            let ephemeral_key = Path::new(&OUT_DIR.to_string()).join("Enclave.pem");
            if !ephemeral_key.exists() {
                let key = Rsa::generate_with_e(3072, &BigNum::from_u32(3).unwrap()).unwrap();
                let pem = key.private_key_to_pem().unwrap();
                fs::write(&ephemeral_key, pem).unwrap();
            }
            ephemeral_key.to_str().unwrap().to_string()
        }
    };
}

fn warn(msg: &str) {
    println!("cargo:warn={}", msg);
}

fn subprocess(program: &str, args: &[&str], current_dir: Option<&PathBuf>) {
    let mut comm = Command::new(program);
    comm.args(args);
    if let Some(dir) = current_dir {
        comm.current_dir(dir);
    }

    warn(&format!("{:#?}", comm));
    match comm.status().unwrap().code() {
        Some(0) => (),
        Some(i) => exit(i),
        None => exit(-1), // Exit caused by signal
    }
}

fn generate_extern_proxy(output: &PathBuf, files: &Vec<PathBuf>) {
    let out_dir = output.parent().unwrap();

    let content = files
        .iter()
        .map(|name| {
            let p = name.strip_prefix(out_dir).unwrap();
            format!(
                r#"include!(concat!(env!("OUT_DIR"), "/{}"));"#,
                p.to_str().unwrap()
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    fs::write(output, &content).unwrap();
}
