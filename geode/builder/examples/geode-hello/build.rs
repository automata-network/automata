
fn main() {
    let build = geode_builder::GeodeBuild::new();
    // println!("{:#?}", build);
    build.build();

    println!("cargo:rerun-if-changed=src/main.rs");
}