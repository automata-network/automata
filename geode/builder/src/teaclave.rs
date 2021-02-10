// SPDX-License-Identifier: Apache-2.0

use home;
use lazy_static::lazy_static;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const GIT_REPO: &'static str = "teaclave-sgx-sdk"; // https://github.com/apache/teaclave-sgx-sdk
const GIT_COMMIT: &'static str = "a6a172e"; // the commit corresponding to tag v1.1.3
pub const RUST_TOOLCHAIN: &'static str = "nightly-2020-10-25"; // the toolchain version required by teaclave-sgx-sdk

lazy_static! {
    pub static ref SGX_SDK: String = {
        println!("cargo:rerun-if-env-changed=RUST_SGX_SDK");
        env::var("RUST_SGX_SDK").unwrap_or_else(|_| {
            find_cargo_git_checkout(GIT_REPO, GIT_COMMIT)
                .expect("environment variable RUST_SGX_SDK not set")
        })
    };
}

pub fn sgx_sdk() -> PathBuf {
    Path::new(&SGX_SDK.to_string()).to_path_buf()
}

pub fn sgx_sdk_includes() -> Vec<PathBuf> {
    vec![sgx_sdk().join("common/inc"), sgx_sdk().join("edl")]
}

pub fn sgx_sdk_untrusted_includes() -> Vec<PathBuf> {
    vec![sgx_sdk().join("edl")]
}

fn find_cargo_git_checkout<'a>(crate_name: &'a str, commit: &'a str) -> Option<String> {
    let git_checkouts = home::cargo_home().ok()?.join("git/checkouts");

    let entries: Vec<PathBuf> = fs::read_dir(git_checkouts)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|name| name.starts_with(crate_name))
                .unwrap_or(false)
        })
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.path())
        .filter_map(|checkout| checkout.read_dir().ok())
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|name| name == commit)
                .unwrap_or(false)
        })
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.path().canonicalize())
        .filter_map(Result::ok)
        .collect();

    entries.first()?.to_str().map(str::to_string)
}
