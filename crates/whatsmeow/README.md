# whatsmeow

Idiomatic, thread-safe Rust bindings for WhatsApp via [WhatsMeow](https://github.com/tulir/whatsmeow).

[![Crates.io](https://img.shields.io/crates/v/whatsmeow.svg)](https://crates.io/crates/whatsmeow)
[![Documentation](https://docs.rs/whatsmeow/badge.svg)](https://docs.rs/whatsmeow)

## Features

- ✨ **Fluent builder API** with async callbacks
- 📡 **Dual event model**: Callbacks or async streams
- 📸 **Media support**: Send images with auto MIME detection
- 🏷️ **Custom device name** in WhatsApp's "Linked Devices"
- 📊 **Memory tracking** for FFI debugging
- 🔒 **Thread-safe** client sharing
- 📦 **Zero manual setup** - Go bridge compiles automatically

## Quick Start

```rust
use whatsmeow::{WhatsApp, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let client = WhatsApp::connect("data/session.db")
        .device_name("My App")
        .on_qr(|qr| async move {
            if let Some(code) = qr.code() {
                println!("Scan: {}", code);
            }
        })
        .on_message(|msg| async move {
            println!("{}: {}", msg.sender_name(), msg.text());
        })
        .build()
        .await?;

    tokio::select! {
        r = client.run() => r?,
        _ = tokio::signal::ctrl_c() => client.disconnect(),
    }
    Ok(())
}
```

## Sending Images

```rust
use whatsmeow::{Jid, MediaSource, MessageType};

// Auto-detect MIME type from file signature
client.send(
    Jid::user("1234567890"),
    MessageType::image_auto(MediaSource::file("photo.jpg"))
)?;

// With caption
client.send(
    Jid::user("1234567890"),
    MessageType::image_auto_with_caption(
        MediaSource::file("photo.png"),
        "Check this out!"
    )
)?;
```

## MediaSource Options

| Source                            | Usage                |
| --------------------------------- | -------------------- |
| `MediaSource::file("path.jpg")`   | Load from local file |
| `MediaSource::bytes(vec![...])`   | Raw bytes            |
| `MediaSource::base64("...")`      | Base64 encoded       |
| `MediaSource::url("https://...")` | Remote URL (future)  |

## Event Types

| Event          | Description            |
| -------------- | ---------------------- |
| `Qr`           | QR code for scanning   |
| `Connected`    | Successfully connected |
| `Disconnected` | Connection lost        |
| `Message`      | Incoming message       |
| `Receipt`      | Delivery/read receipt  |
| `Presence`     | Online/offline status  |

## License

MIT
