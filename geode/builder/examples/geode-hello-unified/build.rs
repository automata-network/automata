
fn main() {
    geode_builder::GeodeBuild::new().build();

    println!("cargo:rerun-if-changed=src/main.rs");
}