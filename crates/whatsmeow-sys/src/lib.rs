//! Raw FFI bindings to the WhatsApp Go bridge DLL
//!
//! ⚠️ **WARNING**: This module contains unsafe code.
//! Use the safe `whatsmeow` crate instead.

#![allow(non_camel_case_types)]

use libc::{c_char, c_int, c_void};

/// Opaque handle to a WhatsApp client instance
pub type ClientHandle = *mut c_void;

/// Result code from FFI operations
pub type WmResult = c_int;

/// Error codes
pub mod error_codes {
    use libc::c_int;

    pub const WM_OK: c_int = 0;
    pub const WM_ERR_INIT: c_int = -1;
    pub const WM_ERR_CONNECT: c_int = -2;
    pub const WM_ERR_DISCONNECTED: c_int = -3;
    pub const WM_ERR_INVALID_HANDLE: c_int = -4;
    pub const WM_ERR_BUFFER_TOO_SMALL: c_int = -5;
}

unsafe extern "C" {
    /// Initialize a new WhatsApp client
    pub fn wm_client_new(db_path: *const c_char) -> ClientHandle;

    /// Connect the client to WhatsApp
    pub fn wm_client_connect(handle: ClientHandle) -> WmResult;

    /// Disconnect and cleanup
    pub fn wm_client_disconnect(handle: ClientHandle) -> WmResult;

    /// Destroy client and free resources
    pub fn wm_client_destroy(handle: ClientHandle);

    /// Poll for next event (non-blocking)
    pub fn wm_poll_event(handle: ClientHandle, buf: *mut c_char, buf_len: c_int) -> c_int;

    /// Send a text message
    pub fn wm_send_message(
        handle: ClientHandle,
        jid: *const c_char,
        text: *const c_char,
    ) -> WmResult;

    /// Get last error message
    pub fn wm_last_error(handle: ClientHandle, buf: *mut c_char, buf_len: c_int) -> c_int;
}
