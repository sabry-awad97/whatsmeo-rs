//! Safe wrappers around FFI bindings

use std::ffi::CString;
use std::path::Path;

use whatsmeow_sys::{self as sys, error_codes::*, ClientHandle};

use crate::error::{Error, Result};

/// Safe wrapper around the raw FFI handle
pub(crate) struct FfiClient {
    handle: ClientHandle,
    event_buffer: Vec<u8>,
}

impl FfiClient {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let path = db_path
            .as_ref()
            .to_str()
            .ok_or_else(|| Error::Init("Invalid path encoding".into()))?;

        let c_path =
            CString::new(path).map_err(|_| Error::Init("Path contains null byte".into()))?;

        let handle = unsafe { sys::wm_client_new(c_path.as_ptr()) };

        if handle.is_null() {
            return Err(Error::Init("Failed to create client".into()));
        }

        Ok(Self {
            handle,
            event_buffer: vec![0u8; 64 * 1024],
        })
    }

    pub fn connect(&self) -> Result<()> {
        let result = unsafe { sys::wm_client_connect(self.handle) };
        self.check_result(result)
    }

    pub fn disconnect(&self) -> Result<()> {
        let result = unsafe { sys::wm_client_disconnect(self.handle) };
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

    pub fn send_message(&self, jid: &str, text: &str) -> Result<()> {
        let c_jid = CString::new(jid).map_err(|_| Error::Send("JID contains null byte".into()))?;
        let c_text =
            CString::new(text).map_err(|_| Error::Send("Text contains null byte".into()))?;

        let result = unsafe { sys::wm_send_message(self.handle, c_jid.as_ptr(), c_text.as_ptr()) };

        self.check_result(result)
    }

    fn check_result(&self, code: i32) -> Result<()> {
        match code {
            WM_OK => Ok(()),
            WM_ERR_INIT => Err(Error::Init("Initialization failed".into())),
            WM_ERR_CONNECT => Err(Error::Connection("Connection failed".into())),
            WM_ERR_DISCONNECTED => Err(Error::Disconnected),
            WM_ERR_INVALID_HANDLE => Err(Error::InvalidHandle),
            _ => Err(Error::Ffi {
                code,
                message: "Unknown error".into(),
            }),
        }
    }
}

impl Drop for FfiClient {
    fn drop(&mut self) {
        unsafe { sys::wm_client_destroy(self.handle) };
    }
}

unsafe impl Send for FfiClient {}
