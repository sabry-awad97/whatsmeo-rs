//! Safe wrappers around FFI bindings

use std::ffi::CString;
use std::path::Path;

use tracing::{debug, warn};
use whatsmeow_sys::{self as sys, ClientHandle, error_codes::*};

use crate::allocator::TrackedAllocator;
use crate::error::{Error, Result};

/// Global allocator reference for tracing (set by the example/app)
#[global_allocator]
static GLOBAL: TrackedAllocator = TrackedAllocator::new();

/// Safe wrapper around the raw FFI handle
pub(crate) struct FfiClient {
    handle: ClientHandle,
    event_buffer: Vec<u8>,
}

impl FfiClient {
    #[tracing::instrument(skip_all, name = "ffi.new", fields(path = %db_path.as_ref().display()))]
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let path = db_path.as_ref();

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
            && !parent.exists()
        {
            debug!(dir = %parent.display(), "Creating parent directory");
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::Init(format!("Failed to create directory: {}", e)))?;
        }

        let path_str = path
            .to_str()
            .ok_or_else(|| Error::Init("Invalid path encoding".into()))?;

        let c_path =
            CString::new(path_str).map_err(|_| Error::Init("Path contains null byte".into()))?;

        let handle = GLOBAL.trace_operation("wm_client_new", || unsafe {
            sys::wm_client_new(c_path.as_ptr())
        });

        if handle.is_null() {
            warn!("FFI returned null handle");
            return Err(Error::Init("Failed to create client".into()));
        }

        debug!("FFI client created successfully");
        Ok(Self {
            handle,
            event_buffer: vec![0u8; 64 * 1024],
        })
    }

    #[tracing::instrument(skip(self), name = "ffi.connect")]
    pub fn connect(&self) -> Result<()> {
        let result = GLOBAL.trace_operation("wm_client_connect", || unsafe {
            sys::wm_client_connect(self.handle)
        });
        self.check_result(result)
    }

    #[tracing::instrument(skip(self), name = "ffi.disconnect")]
    pub fn disconnect(&self) -> Result<()> {
        let result = GLOBAL.trace_operation("wm_client_disconnect", || unsafe {
            sys::wm_client_disconnect(self.handle)
        });
        self.check_result(result)
    }

    pub fn poll_event(&mut self) -> Result<Option<Vec<u8>>> {
        let n = unsafe {
            sys::wm_poll_event(
                self.handle,
                self.event_buffer.as_mut_ptr() as *mut i8,
                self.event_buffer.len() as i32,
            )
        };

        if n < 0 {
            self.check_result(n)?;
        }

        if n == 0 {
            return Ok(None);
        }

        Ok(Some(self.event_buffer[..n as usize].to_vec()))
    }

    #[tracing::instrument(skip(self), name = "ffi.send_message", fields(to = %jid, text_len = text.len()))]
    pub fn send_message(&self, jid: &str, text: &str) -> Result<()> {
        let c_jid = CString::new(jid).map_err(|_| Error::Send("JID contains null byte".into()))?;
        let c_text =
            CString::new(text).map_err(|_| Error::Send("Text contains null byte".into()))?;

        let result = GLOBAL.trace_operation("wm_send_message", || unsafe {
            sys::wm_send_message(self.handle, c_jid.as_ptr(), c_text.as_ptr())
        });

        self.check_result(result)
    }

    fn check_result(&self, code: i32) -> Result<()> {
        match code {
            WM_OK => Ok(()),
            WM_ERR_INIT => {
                warn!(code, "FFI initialization error");
                Err(Error::Init("Initialization failed".into()))
            }
            WM_ERR_CONNECT => {
                warn!(code, "FFI connection error");
                Err(Error::Connection("Connection failed".into()))
            }
            WM_ERR_DISCONNECTED => {
                debug!("FFI reports disconnected");
                Err(Error::Disconnected)
            }
            WM_ERR_INVALID_HANDLE => {
                warn!(code, "FFI invalid handle");
                Err(Error::InvalidHandle)
            }
            _ => {
                warn!(code, "FFI unknown error");
                Err(Error::Ffi {
                    code,
                    message: "Unknown error".into(),
                })
            }
        }
    }
}

impl Drop for FfiClient {
    fn drop(&mut self) {
        GLOBAL.trace_operation("wm_client_destroy", || unsafe {
            sys::wm_client_destroy(self.handle)
        });

        GLOBAL.print_stats();
    }
}

unsafe impl Send for FfiClient {}
