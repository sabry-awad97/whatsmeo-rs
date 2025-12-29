# whatsmeow-sys

Raw FFI bindings to the WhatsApp Go bridge via [WhatsMeow](https://github.com/tulir/whatsmeow).

> ⚠️ **Low-level crate** - Use [`whatsmeow`](https://crates.io/crates/whatsmeow) for the safe, idiomatic API.

## FFI Functions

```rust
// Create client with custom device name
wm_client_new(db_path, device_name) -> ClientHandle

// Connection
wm_client_connect(handle) -> WmResult
wm_client_disconnect(handle) -> WmResult
wm_client_destroy(handle)

// Events
wm_poll_event(handle, buf, buf_len) -> c_int

// Messaging
wm_send_message(handle, jid, text) -> WmResult
wm_send_image(handle, jid, data, data_len, mime_type, caption) -> WmResult
```

## Error Codes

| Code | Constant                  | Meaning               |
| ---- | ------------------------- | --------------------- |
| 0    | `WM_OK`                   | Success               |
| -1   | `WM_ERR_INIT`             | Initialization failed |
| -2   | `WM_ERR_CONNECT`          | Connection failed     |
| -3   | `WM_ERR_DISCONNECTED`     | Client disconnected   |
| -4   | `WM_ERR_INVALID_HANDLE`   | Invalid handle        |
| -5   | `WM_ERR_BUFFER_TOO_SMALL` | Buffer too small      |

## Requirements

- **Go 1.21+** with CGO enabled
- **Windows**: MSVC toolchain
- **Linux/macOS**: GCC or Clang

## License

MIT
