use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let go_dir = manifest_dir.join("go");
    let go_bridge_dir = go_dir.join("bridge");

    // Use OUT_DIR for artifacts when built as a dependency
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let go_target_dir = out_dir.join("go_target");

    if !go_target_dir.exists() {
        std::fs::create_dir_all(&go_target_dir).expect("failed to create go target directory");
    }

    // 1. Ensure Go bridge is built
    build_go_bridge(&go_bridge_dir, &go_target_dir);

    // 2. Configure linker
    println!("cargo:rustc-link-search=native={}", go_target_dir.display());
    println!("cargo:rustc-link-lib=dylib=whatsmeow");

    // Re-run build script if Go bridge files change
    println!("cargo:rerun-if-changed={}", go_bridge_dir.display());
}

fn build_go_bridge(bridge_dir: &Path, target_dir: &Path) {
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let dll_name = if os == "windows" {
        "whatsmeow.dll"
    } else {
        "whatsmeow.so"
    };

    let dll_path = target_dir.join(dll_name);

    println!("cargo:warning=🏗️ Building Go bridge (CGO)...");

    let mut cmd = Command::new("go");
    cmd.arg("build")
        .arg("-buildmode=c-shared")
        .arg("-o")
        .arg(&dll_path)
        .arg(".")
        .current_dir(bridge_dir)
        .env("CGO_ENABLED", "1");

    let status = cmd.status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => panic!(
            "Go bridge build failed with status: {}. Ensure Go 1.21+ is installed.",
            s
        ),
        Err(e) => panic!(
            "Failed to execute 'go' command: {}. Is Go installed and in PATH?",
            e
        ),
    }

    // On Windows, we need the .lib import library for MSVC linking
    if os == "windows" {
        generate_msvc_import_lib(target_dir);
    }
}

fn generate_msvc_import_lib(target_dir: &Path) {
    let lib_path = target_dir.join("whatsmeow.lib");
    if lib_path.exists() {
        // Still try to regenerate if DLL might be newer, but for now skip if exists
        // but generate_lib.ps1 is fast enough.
    }

    println!("cargo:warning=🔧 Generating MSVC import library...");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let script_path = manifest_dir
        .join("go")
        .join("scripts")
        .join("generate_lib.ps1");

    let mut cmd = Command::new("powershell");
    cmd.arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-File")
        .arg(&script_path)
        .arg("-TargetDir")
        .arg(target_dir);

    let status = cmd.status().expect("failed to execute generate_lib.ps1");

    if !status.success() {
        panic!("MSVC import library generation failed. Ensure Visual Studio with C++ tools is installed.");
    }
}
