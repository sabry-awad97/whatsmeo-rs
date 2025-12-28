//! Embedded DLL loader for self-contained binaries
//!
//! Embeds the Go bridge DLL at compile time and extracts it at runtime.
//! Enable with: `cargo build --features embed-dll`

#[cfg(feature = "embed-dll")]
mod inner {
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::Once;

    /// Embedded DLL bytes (included at compile time)
    #[cfg(target_os = "windows")]
    static DLL_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/go_target/whatsmeow.dll"));

    #[cfg(not(target_os = "windows"))]
    static DLL_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/go_target/whatsmeow.so"));

    static EXTRACT_ONCE: Once = Once::new();
    static mut EXTRACTED_PATH: Option<PathBuf> = None;

    /// Get the path to the extracted DLL, extracting it if necessary
    pub fn get_dll_path() -> &'static PathBuf {
        EXTRACT_ONCE.call_once(|| {
            let path = extract_dll().expect("Failed to extract embedded DLL");
            unsafe {
                EXTRACTED_PATH = Some(path);
            }
        });
        unsafe { EXTRACTED_PATH.as_ref().unwrap() }
    }

    /// Extract the embedded DLL to a temporary location
    fn extract_dll() -> std::io::Result<PathBuf> {
        let dll_dir = get_dll_directory()?;
        fs::create_dir_all(&dll_dir)?;

        #[cfg(target_os = "windows")]
        let dll_name = "whatsmeow.dll";
        #[cfg(not(target_os = "windows"))]
        let dll_name = "whatsmeow.so";

        let dll_path = dll_dir.join(dll_name);

        // Check if DLL already exists with correct size
        if dll_path.exists() {
            if let Ok(metadata) = fs::metadata(&dll_path) {
                if metadata.len() == DLL_BYTES.len() as u64 {
                    tracing::debug!(path = %dll_path.display(), "Using cached embedded DLL");
                    return Ok(dll_path);
                }
            }
        }

        tracing::info!(path = %dll_path.display(), "Extracting embedded DLL");
        let mut file = fs::File::create(&dll_path)?;
        file.write_all(DLL_BYTES)?;
        file.sync_all()?;

        Ok(dll_path)
    }

    fn get_dll_directory() -> std::io::Result<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            if let Ok(app_data) = std::env::var("LOCALAPPDATA") {
                Ok(PathBuf::from(app_data).join("whatsmeow-rs").join("lib"))
            } else {
                Ok(std::env::temp_dir().join("whatsmeow-rs").join("lib"))
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            Ok(std::env::temp_dir().join("whatsmeow-rs").join("lib"))
        }
    }

    /// Ensure the DLL is extracted and loadable
    pub fn ensure_dll_extracted() {
        let path = get_dll_path();
        tracing::debug!(path = %path.display(), "DLL path ready");
    }
}

#[cfg(feature = "embed-dll")]
pub use inner::ensure_dll_extracted;

/// No-op when embed-dll feature is disabled
#[cfg(not(feature = "embed-dll"))]
pub fn ensure_dll_extracted() {
    // DLL is loaded from system path when not embedded
}
