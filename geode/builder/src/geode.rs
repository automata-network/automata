// SPDX-License-Identifier: Apache-2.0

use crate::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Metadata {
    geode: Config,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Config {
    enclaves: Vec<String>,
}

#[derive(Debug)]
pub struct Build {
    manifest_dir: PathBuf,
    out_dir: PathBuf,
    enclaves: Vec<enclave::Build>,
}

impl Default for geode::Build {
    fn default() -> Self {
        Self::new()
    }
}

impl Build {
    pub fn new() -> Build {
        let manifest_dir = Path::new(&CARGO_MANIFEST_DIR.to_string()).to_path_buf();
        Build::from_manifest(&manifest_dir)
    }

    pub fn from_manifest(manifest_dir: &PathBuf) -> Build {
        let manifest =
            Manifest::<Metadata>::from_path_with_metadata(manifest_dir.join("Cargo.toml")).unwrap();
        let out_dir = Path::new(&OUT_DIR.to_string()).to_path_buf();

        let enclaves = manifest.package.map_or(vec![], |p| {
            p.metadata.map_or(vec![], |m| {
                m.geode
                    .enclaves
                    .iter()
                    .map(|dir| {
                        enclave::Build::from_manifest(&manifest_dir.join(dir), Some(&out_dir))
                    })
                    .collect()
            })
        });

        Build {
            manifest_dir: manifest_dir.to_path_buf(),
            out_dir,
            enclaves,
        }
    }

    pub fn build(&self) {
        let mut ecall_externs = vec![];

        self.enclaves.iter().for_each(|e| {
            e.build_crate();
            e.generate_interfaces();
            ecall_externs.extend(e.collect_ecall_extern());
            e.build_enclave();
            e.sign_enclave();
            e.build_untrusted();

            let enclave_path = e.signed_enclave_path();
            let target_path = self.out_dir.join(enclave_path.file_name().unwrap());

            // FIXME: get the path heuristically instead of hard-coding
            let target_path2 = self
                .out_dir
                .join("../../../")
                .join(enclave_path.file_name().unwrap());

            fs::copy(enclave_path, &target_path).unwrap();
            fs::copy(enclave_path, &target_path2).unwrap();
        });

        generate_extern_proxy(&self.out_dir.join("ecall.rs"), &ecall_externs);

        intel::sgx_sdk_cargo_metadata();
    }
}
