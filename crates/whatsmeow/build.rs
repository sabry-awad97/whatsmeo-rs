fn main() {
    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();
    let go_target_dir = workspace_root.join("go").join("target");

    #[cfg(target_os = "windows")]
    let lib_name = "whatsmeow.lib";
    #[cfg(not(target_os = "windows"))]
    let lib_name = "whatsmeow.so";

    let lib_path = go_target_dir.join(lib_name);

    if !lib_path.exists() {
        println!(
            "cargo:warning=⚠️  Could not find Go bridge library at: {}",
            lib_path.display()
        );
        println!("cargo:warning=💡 Please run 'task build' to compile the Go bridge first.");
    }
}
