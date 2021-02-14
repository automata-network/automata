// SPDX-License-Identifier: Apache-2.0

use std::vec;

use crate::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Metadata {
    enclave: Config,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Config {
    config: String,
    edl: String,
    linker_script: String,
}

#[derive(Debug)]
pub struct Build {
    crate_name: String,
    manifest_dir: PathBuf,
    out_dir: PathBuf,
    trusted_dir: PathBuf,
    untrusted_dir: PathBuf,
    config_path: PathBuf,
    edl_path: PathBuf,
    linker_script_path: PathBuf,
    toolchain: String,
    enclave_path: PathBuf,
    signed_enclave_path: PathBuf,
    teaclave_dir: PathBuf,
    local_dependencies: Vec<PathBuf>,
}

impl Default for enclave::Build {
    fn default() -> Self {
        Self::new()
    }
}

impl Build {
    /// Create new build when used standalone
    pub fn new() -> Build {
        let manifest_dir = Path::new(&CARGO_MANIFEST_DIR.to_string()).to_path_buf();
        Build::from_manifest(&manifest_dir, None)
    }

    pub fn from_manifest(manifest_dir: &PathBuf, geode_out_dir: Option<&PathBuf>) -> Build {
        let manifest_dir = manifest_dir.canonicalize().unwrap();

        let manifest =
            Manifest::<Metadata>::from_path_with_metadata(&manifest_dir.join("Cargo.toml"))
                .unwrap();

        let package = manifest.package.unwrap();
        // TODO: need to handle parse failure
        let metadata = &package.metadata.unwrap();

        let local_dependencies = manifest
            .dependencies
            .iter()
            .filter_map(|(_, dep)| dep.detail().and_then(|detail| detail.path.as_ref()))
            .filter_map(|path| manifest_dir.join(path).canonicalize().ok())
            .collect::<Vec<PathBuf>>();

        let crate_name = package.name.replace("-", "_");
        let out_dir = match geode_out_dir {
            Some(geode) => geode.join(&package.name),
            None => Path::new(&OUT_DIR.to_string()).to_path_buf(),
        };

        // Path::new(&OUT_DIR.to_string()).join(&package.name);
        let trusted_dir = out_dir.join("trusted");
        let untrusted_dir = out_dir.join("untrusted");

        let config_path = manifest_dir.join(&metadata.enclave.config);
        let edl_path = manifest_dir.join(&metadata.enclave.edl);
        let linker_script_path = manifest_dir.join(&metadata.enclave.linker_script);
        let enclave_path = out_dir.join(&format!("{}.so", &crate_name));
        let signed_enclave_path = out_dir.join(&format!("{}.signed.so", &crate_name));

        let toolchain = fs::read_to_string(&manifest_dir.join("rust-toolchain"))
            .map_or(teaclave::RUST_TOOLCHAIN.to_string(), |s| {
                s.trim().to_string()
            });

        let teaclave_dir = out_dir.join("teaclave");

        Build {
            crate_name,
            out_dir,
            trusted_dir,
            untrusted_dir,
            manifest_dir,
            teaclave_dir,
            config_path,
            edl_path,
            linker_script_path,
            toolchain,
            enclave_path,
            signed_enclave_path,
            local_dependencies,
        }
    }

    /// Build only the basic components for an enclave crate
    pub fn build(&self) {
        self.generate_interfaces();
        self.generate_ocall_extern_proxy();
    }

    pub fn build_crate(&self) {
        let manifest_path = self.manifest_dir.join("Cargo.toml");

        subprocess(
            "cargo",
            &[
                &format!("+{}", &self.toolchain), // make sure the right toolchain is used to compile enclave
                "-Z",
                "unstable-options",
                "build",
                "--manifest-path",
                &manifest_path.to_str().unwrap(),
                "--profile",
                &PROFILE.to_string(),
                "--color",
                "always",
                "--out-dir",
                &OUT_DIR.to_string(),
                "--target-dir",
                &self.trusted_dir.to_str().unwrap(),
            ],
            None,
        );

        self.rerun_if_changed();
    }

    pub fn generate_interfaces(&self) {
        intel::Edger8r::new()
            .file(&self.edl_path)
            .trusted(&self.trusted_dir)
            .untrusted(&self.untrusted_dir)
            .search_path(&intel::sgx_sdk().join("include"))
            .search_path(&teaclave::sgx_sdk().join("edl"))
            .use_customized()
            .teaclave(&self.teaclave_dir)
            .run();
    }

    pub fn collect_ecall_extern(&self) -> Vec<PathBuf> {
        self.collect_extern("ecall.rs")
    }

    fn collect_extern(&self, proxy_name: &str) -> Vec<PathBuf> {
        let mut extern_files = vec![];
        for entry in fs::read_dir(&self.teaclave_dir).unwrap() {
            let entry = entry.unwrap();
            let file_name = entry.file_name().into_string().unwrap();
            if file_name.ends_with(&proxy_name) {
                extern_files.push(entry.path());
            }
        }
        extern_files
    }

    // Generate the ocall proxy file
    fn generate_ocall_extern_proxy(&self) {
        let proxy_name = "ocall.rs";
        let files = self.collect_extern(proxy_name);

        generate_extern_proxy(&self.out_dir.join(proxy_name), &files);
    }

    pub fn build_enclave(&self) {
        let interface_file = self.edl_path.file_stem().unwrap().to_str().unwrap();
        let interface_source = self
            .trusted_dir
            .join(format!("{}_t.c", &interface_file))
            .to_str()
            .unwrap()
            .to_string();

        let mut cc = cc::Build::new();

        cc.cpp(true).no_default_flags(true).cargo_metadata(false);

        // Tell c++ to treat the interface source file as C file
        cc.flag("-xc");
        cc.flag(&interface_source);

        cc.includes(teaclave::sgx_sdk_includes());
        cc.includes(intel::sgx_sdk_includes());

        cc.flag("-nostdinc")
            .flag("-fvisibility=hidden")
            .flag("-fpie")
            .flag("-fstack-protector")
            .flag("-nostdlib")
            .flag("-nodefaultlibs")
            .flag("-nostartfiles");

        for flag in &intel::sgx_common_cflags() {
            cc.flag(flag);
        }

        // linker flags
        cc.flag(&format!(
            "-L{}",
            intel::sgx_sdk_lib_path().to_str().unwrap().to_string()
        ));
        cc.flag(&format!("-L{}", &OUT_DIR.to_string()));

        cc.flag("-Wl,--no-undefined");
        cc.flag("-Wl,--whole-archive");
        intel::sgx_sdk_whole_archive_libs().iter().for_each(|lib| {
            cc.flag(&format!("-l{}", lib));
        });
        cc.flag("-Wl,--no-whole-archive");
        cc.flag("-Wl,--start-group");
        cc.flag(&format!("-l{}", &self.crate_name));
        intel::sgx_sdk_libs().iter().for_each(|lib| {
            cc.flag(&format!("-l{}", lib));
        });
        cc.flag("-Wl,--end-group");
        cc.flag("-Wl,-Bstatic");
        cc.flag("-Wl,-Bsymbolic");
        cc.flag("-Wl,--no-undefined");
        cc.flag("-Wl,-pie,-eenclave_entry");
        cc.flag("-Wl,--export-dynamic");
        cc.flag("-Wl,--defsym,__ImageBase=0");
        cc.flag("-Wl,--gc-sections");
        cc.flag(&format!(
            "-Wl,--version-script={}",
            &self.linker_script_path.to_str().unwrap().to_string()
        ));

        cc.flag("-o");
        cc.flag(&self.enclave_path.to_str().unwrap().to_string());

        println!("{:?}", cc.get_compiler().to_command());

        let mut command = cc.get_compiler().to_command();

        match command.status().unwrap().code() {
            Some(0) => (),
            Some(i) => exit(i),
            None => exit(-1), // Exit caused by signal
        }
    }

    pub fn sign_enclave(&self) {
        let key = Path::new(&ENCLAVE_SIGNING_KEY.to_string()).to_path_buf();

        intel::Sign::new()
            .config(&self.config_path)
            .enclave(&self.enclave_path)
            .out(&self.signed_enclave_path)
            .key(&key)
            .run();
    }

    pub fn signed_enclave_path(&self) -> &PathBuf {
        &self.signed_enclave_path
    }

    pub fn build_untrusted(&self) {
        let interface_file = self.edl_path.file_stem().unwrap().to_str().unwrap();
        let interface_source = self.untrusted_dir.join(format!("{}_u.c", interface_file));

        cc::Build::new()
            .file(&interface_source)
            .include(&self.untrusted_dir)
            .includes(teaclave::sgx_sdk_untrusted_includes())
            .includes(intel::sgx_sdk_untrusted_includes())
            .flag("-fPIC")
            .flag("-Wno-attributes")
            .compile(interface_source.file_stem().unwrap().to_str().unwrap());
    }

    fn rerun_if_changed(&self) {
        let mut watch_dirs = vec![self.manifest_dir.to_owned()];
        watch_dirs.extend(self.local_dependencies.to_owned());

        for path in watch_dirs {
            Build::walk_manifest_dir(&path).unwrap_or_else(|e| {
                warn(&format!("{:#?}", e));
            })
        }
    }

    fn walk_manifest_dir(dir: &PathBuf) -> io::Result<()> {
        let print_rerun = |p: &PathBuf| {
            if let Some(s) = p.to_str() {
                println!("cargo:rerun-if-changed={}", s);
            }
        };

        let handle_entry = |e: fs::DirEntry| -> Result<(), _> {
            let file_name = e.file_name();
            let p = e.path();

            if p.is_dir() && file_name != "target" {
                Build::walk_manifest_dir(&p)?;
                print_rerun(&p);
                return Ok(());
            }

            if p.is_file() {
                print_rerun(&p);
                return Ok(());
            }

            Ok(())
        };

        for entry in fs::read_dir(dir)? {
            entry
                .and_then(handle_entry)
                .unwrap_or_else(|e| warn(&format!("{:#?}", e)));
        }
        Ok(())
    }
}
