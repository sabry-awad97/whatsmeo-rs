fn main() {
    // Link to the Go-built DLL
    if let Ok(lib_dir) = std::env::var("WHATSMEOW_LIB_DIR") {
        println!("cargo:rustc-link-search=native={}", lib_dir);
    }

    println!("cargo:rustc-link-lib=dylib=whatsmeow");
    println!("cargo:rerun-if-env-changed=WHATSMEOW_LIB_DIR");
}
