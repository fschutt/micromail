use std::env;
use std::path::PathBuf;

fn main() {
    // Generate C header file
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    
    let output_file = target_dir()
        .join(format!("{}.h", package_name.replace("-", "_")))
        .display()
        .to_string();
    
    let include_dir = PathBuf::from(&crate_dir).join("include");
    std::fs::create_dir_all(&include_dir).unwrap();
    
    let config = cbindgen::Config::from_file("cbindgen.toml")
        .expect("Unable to find cbindgen.toml");
    
    cbindgen::generate_with_config(&crate_dir, config)
        .expect("Unable to generate bindings")
        .write_to_file("include/micromail.h");
}

fn target_dir() -> PathBuf {
    let out_dir = env::var("OUT_DIR").unwrap();
    PathBuf::from(out_dir)
}