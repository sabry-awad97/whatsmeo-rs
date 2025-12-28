# whatsmeow-sys

Raw FFI bindings to the WhatsApp Go bridge via [WhatsMeow](https://github.com/tulir/whatsmeow).

> ⚠️ **Low-level crate** - Use [`whatsmeow`](https://crates.io/crates/whatsmeow) for the safe, idiomatic API.

## Features

- 🔗 C-compatible FFI bindings to Go WhatsMeow library
- 🏗️ **Automatic Go bridge compilation** via `build.rs`
- 📦 Go source included in crate for seamless builds

## FFI Functions

```rust
// Create client with custom device name
wm_client_new(db_path: *const c_char, device_name: *const c_char) -> ClientHandle

// Connection management
wm_client_connect(handle: ClientHandle) -> WmResult
wm_client_disconnect(handle: ClientHandle) -> WmResult
wm_client_destroy(handle: ClientHandle)

// Event polling (non-blocking)
wm_poll_event(handle: ClientHandle, buf: *mut c_char, buf_len: c_int) -> c_int

// Send message
wm_send_message(handle: ClientHandle, jid: *const c_char, text: *const c_char) -> WmResult
```

## Error Codes

| Code | Constant                  | Meaning               |
| ---- | ------------------------- | --------------------- |
| 0    | `WM_OK`                   | Success               |
| -1   | `WM_ERR_INIT`             | Initialization failed |
| -2   | `WM_ERR_CONNECT`          | Connection failed     |
| -3   | `WM_ERR_DISCONNECTED`     | Client disconnected   |
| -4   | `WM_ERR_INVALID_HANDLE`   | Invalid client handle |
| -5   | `WM_ERR_BUFFER_TOO_SMALL` | Buffer too small      |

## Requirements

- **Go 1.24+** with CGO enabled
- **Windows**: MSVC toolchain
- **Linux/macOS**: GCC or Clang

## License

MIT
