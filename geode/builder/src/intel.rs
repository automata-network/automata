// SPDX-License-Identifier: Apache-2.0

use crate::subprocess;
use crate::OUT_DIR;
use crate::PROFILE;
use lazy_static::lazy_static;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

lazy_static! {
    pub static ref SGX_MODE: String = {
        println!("cargo:rerun-if-env-changed=SGX_MODE");
        env::var("SGX_MODE").unwrap_or(String::from("HW"))
    };
    pub static ref SGX_SDK: String = {
        println!("cargo:rerun-if-env-changed=SGX_SDK");
        env::var("SGX_SDK").unwrap_or(String::from("/opt/intel/sgxsdk"))
    };
    pub static ref SGX_COMMON_CFLAGS: Vec<&'static str> = {
        let mut args = vec!["-m64"];
        match &PROFILE.to_string() == "release" {
            true => args.push("-O2"),
            false => args.extend(&["-O0", "-g"]),
        }
        args
    };
}

pub fn sgx_sdk() -> PathBuf {
    Path::new(&SGX_SDK.to_string()).to_path_buf()
}

pub fn sgx_sdk_untrusted_includes() -> Vec<PathBuf> {
    vec![sgx_sdk().join("include")]
}

pub fn sgx_sdk_includes() -> Vec<PathBuf> {
    vec![
        sgx_sdk().join("include"),
        sgx_sdk().join("include/tlibc"),
        sgx_sdk().join("include/stlport"),
        sgx_sdk().join("include/epid"),
    ]
}

pub fn sgx_sdk_lib_path() -> PathBuf {
    sgx_sdk().join("lib64")
}

pub fn sgx_sdk_whole_archive_libs() -> Vec<&'static str> {
    if SGX_MODE.to_string() == "HW" {
        vec!["sgx_trts", "sgx_tservice"]
    } else {
        vec!["sgx_trts_sim", "sgx_tservice_sim"]
    }
}

pub fn sgx_sdk_libs() -> Vec<&'static str> {
    vec![
        "sgx_tcxx",
        "sgx_tstdc",
        "sgx_tcrypto",
        "sgx_ukey_exchange",
        "sgx_tkey_exchange",
        "sgx_tprotected_fs",
    ]
}

pub fn sgx_common_cflags() -> Vec<&'static str> {
    let mut args = vec!["-m64"];
    match &PROFILE.to_string() == "release" {
        true => args.push("-O2"),
        false => args.extend(&["-O0", "-g"]),
    }
    args
}

pub fn sgx_sdk_cargo_metadata() {
    println!(
        "cargo:rustc-link-search=native={}/lib64",
        &SGX_SDK.to_string()
    );
    println!("cargo:rustc-link-lib=static=sgx_uprotected_fs");
    println!("cargo:rustc-link-lib=static=sgx_ukey_exchange");

    if "HW" == SGX_MODE.to_string() {
        println!("cargo:rustc-link-lib=dylib=sgx_urts");
        println!("cargo:rustc-link-lib=dylib=sgx_epid");
        println!("cargo:rustc-link-lib=dylib=sgx_launch");
        println!("cargo:rustc-link-lib=dylib=sgx_quote_ex");
    } else {
        println!("cargo:rustc-link-lib=dylib=sgx_urts_sim");
        println!("cargo:rustc-link-lib=dylib=sgx_uae_service_sim")
    }
}

pub struct Edger8rBuild {
    pic_support: bool,
    source_dir: PathBuf,
    build_dir: PathBuf,
}

impl Edger8rBuild {
    pub fn new(source_dir: &PathBuf, build_dir: &PathBuf) -> Self {
        Edger8rBuild {
            pic_support: false,
            source_dir: source_dir.to_path_buf(),
            build_dir: build_dir.to_path_buf(),
        }
    }

    pub fn pic_support(&mut self, b: bool) -> &mut Self {
        self.pic_support = b;
        self
    }

    pub fn execute(&self) {
        copy_file_tree(&self.source_dir, &self.build_dir).unwrap();

        let pic_args = "-cflags -ccopt,-fpie -lflags -runtime-variant,_pic,-ccopt,-pie,-ccopt -lflag -Wl,-z,now -no-links -libs str,unix Edger8r.native".split(" ").filter(|s| !s.is_empty()).collect::<Vec<&str>>();

        let args = "-lflag -ccopt -lflag Wl,-z,now -no-links -libs str,unix Edger8r.native"
            .split(" ")
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>();

        if self.pic_support {
            subprocess("ocamlbuild", &pic_args, Some(&self.build_dir));
        } else {
            subprocess("ocamlbuild", &args, Some(&self.build_dir));
        }
    }

    pub fn binary_path(&self) -> PathBuf {
        self.build_dir.join("_build/Edger8r.native")
    }
}

#[derive(Default)]
pub struct Edger8r {
    binary_path: PathBuf,
    args: Vec<String>,
    trusted: bool,
    untrusted: bool,
    teaclave: Option<PathBuf>,
}

impl Edger8r {
    pub fn new() -> Edger8r {
        let binary_path = sgx_sdk().join("bin/x64/sgx_edger8r");
        Edger8r {
            binary_path,
            ..Default::default()
        }
    }

    pub fn use_customized(&mut self) -> &mut Self {
        // build and install to target
        let (major, minor) = get_ocaml_version();
        let pic_support = !(major < 4 || (major == 4 && minor < 2));

        let source_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("edger8r");
        let build_dir = Path::new(&OUT_DIR.to_string()).join("edger8r");

        let mut builder = Edger8rBuild::new(&source_dir, &build_dir);
        builder.pic_support(pic_support);
        builder.execute();

        self.binary_path = builder.binary_path();
        self
    }

    pub fn file(&mut self, file_path: &PathBuf) -> &mut Self {
        self.args.push(file_path.to_str().unwrap().to_string());
        self
    }

    #[allow(dead_code)]
    pub fn binary(&mut self, binary_path: &PathBuf) -> &mut Self {
        self.binary_path = binary_path.to_path_buf();
        self
    }

    pub fn trusted(&mut self, dir: &PathBuf) -> &mut Self {
        self.trusted = true;
        self.args.push("--trusted-dir".to_string());
        self.args.push(dir.to_str().unwrap().to_string());
        self
    }

    pub fn untrusted(&mut self, dir: &PathBuf) -> &mut Self {
        self.untrusted = true;
        self.args.push("--untrusted-dir".to_string());
        self.args.push(dir.to_str().unwrap().to_string());
        self
    }

    pub fn search_path(&mut self, dir: &PathBuf) -> &mut Self {
        self.args.push("--search-path".to_string());
        self.args.push(dir.to_str().unwrap().to_string());
        self
    }

    pub fn teaclave(&mut self, dir: &PathBuf) -> &mut Self {
        self.teaclave = Some(dir.to_path_buf());
        self.args.push("--teaclave-dir".to_string());
        self.args.push(dir.to_str().unwrap().to_string());
        self
    }

    pub fn run(&mut self) {
        let edger8r = self.binary_path.to_str().unwrap();
        if self.trusted {
            self.args.push("--trusted".to_string());
        }
        if self.untrusted {
            self.args.push("--untrusted".to_string());
        }
        if let Some(dir) = &self.teaclave {
            self.args.push("--teaclave".to_string());
            fs::create_dir_all(dir).unwrap();
        }

        let mut args: Vec<&str> = vec![];
        for a in &self.args {
            args.push(a);
        }

        subprocess(edger8r, &args, None);
    }
}

#[derive(Default)]
pub struct Sign {
    binary_path: PathBuf,
    args: Vec<String>,
}

impl Sign {
    pub fn new() -> Sign {
        let binary_path = sgx_sdk().join("bin/x64/sgx_sign");
        Sign {
            binary_path,
            args: vec!["sign".to_string()],
        }
    }

    pub fn key(&mut self, file_path: &PathBuf) -> &mut Self {
        self.args.push("-key".to_string());
        self.args.push(file_path.to_str().unwrap().to_string());
        self
    }

    pub fn enclave(&mut self, file_path: &PathBuf) -> &mut Self {
        self.args.push("-enclave".to_string());
        self.args.push(file_path.to_str().unwrap().to_string());
        self
    }

    pub fn config(&mut self, file_path: &PathBuf) -> &mut Self {
        self.args.push("-config".to_string());
        self.args.push(file_path.to_str().unwrap().to_string());
        self
    }

    pub fn out(&mut self, file_path: &PathBuf) -> &mut Self {
        self.args.push("-out".to_string());
        self.args.push(file_path.to_str().unwrap().to_string());
        self
    }

    pub fn run(&self) {
        let edger8r = self.binary_path.to_str().unwrap();
        let mut args: Vec<&str> = vec![];
        for a in &self.args {
            args.push(a);
        }

        subprocess(edger8r, &args, None);
    }
}

fn get_ocaml_version() -> (u64, u64) {
    let output = Command::new("ocamlopt").arg("-version").output().unwrap();

    if !output.status.success() {
        panic!(
            "Cannot get ocaml version, with error '{}'",
            str::from_utf8(&output.stderr).unwrap()
        );
    }

    let stdout = str::from_utf8(&output.stdout).unwrap();

    let mut splits = stdout
        .split(".")
        .map(|x| x.parse::<u64>())
        .filter_map(|x| x.ok())
        .take(2);

    (splits.next().unwrap(), splits.next().unwrap())
}

/// Copy the entire directory from `src` to `dst`
///
/// Any destination file will be overwritten. If there's any error, there will
/// be no cleanup on the destination.
fn copy_file_tree(src: &Path, dest: &Path) -> io::Result<()> {
    fs::create_dir_all(&dest)?;
    for src_entry in fs::read_dir(src)? {
        let src_entry = src_entry?;
        if src_entry.file_name() == ".git" {
            continue;
        }

        let src_path = src_entry.path();
        let dest_path = dest.join(src_entry.file_name());

        // println!("copying: {:#?} -> {:#?}", src_path, dest_path);
        if src_path.is_dir() {
            fs::create_dir_all(&dest_path)?;
            // println!("copying directory: {:#?} -> {:#?}", src_path, dest_path);
            copy_file_tree(&src_path, &dest_path)?;
        } else {
            if dest_path.exists() {
                fs::remove_file(&dest_path).unwrap();
            }
            fs::copy(&src_path, &dest_path)?;
            // println!("copied {:#?} -> {:#?}", src_path, dest_path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use intel::Edger8r;

    use crate::*;

    #[test]
    #[ignore = "side effect"]
    fn build_customized_edger8r() {
        let mut temp_dir = env::temp_dir();
        temp_dir.push("geode-builder-edger8r-test");
        env::set_var("OUT_DIR", &temp_dir.to_str().unwrap().to_string());
        let mut edger8r = Edger8r::new();
        edger8r.use_customized();
    }
}
