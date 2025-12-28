fn main() {
    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();
    let default_lib_dir = workspace_root.join("go").join("target");

    // Link to the Go-built DLL
    if let Ok(lib_dir) = std::env::var("WHATSMEOW_LIB_DIR") {
        println!("cargo:rustc-link-search=native={}", lib_dir);
    } else if default_lib_dir.exists() {
        println!(
            "cargo:rustc-link-search=native={}",
            default_lib_dir.display()
        );
    }

    println!("cargo:rustc-link-lib=dylib=whatsmeow");
    println!("cargo:rerun-if-env-changed=WHATSMEOW_LIB_DIR");
}
