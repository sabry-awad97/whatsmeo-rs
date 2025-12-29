# whatsmeow-rs 🦀💬

Idiomatic, thread-safe Rust bindings for WhatsApp via [WhatsMeow](https://github.com/tulir/whatsmeow) Go bridge.

[![Crates.io](https://img.shields.io/crates/v/whatsmeow.svg)](https://crates.io/crates/whatsmeow)
[![Documentation](https://docs.rs/whatsmeow/badge.svg)](https://docs.rs/whatsmeow)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Features

- ✨ **Fluent builder API** with async callbacks
- 📡 **Callback & Stream events** - Choose your style
- 📸 **Media support** - Send images with auto MIME detection
- 🔒 **Thread-safe** - Share clients across tasks
- 🏷️ **Custom device name** - Shows in WhatsApp's "Linked Devices"
- 📊 **Tracing integration** - Structured logging with FFI timing
- 📦 **Automated Go bridge** - Compiles automatically via `build.rs`

## Installation

```toml
[dependencies]
whatsmeow = "0.1.4"
tokio = { version = "1", features = ["full"] }
```

> **Note**: Requires Go 1.21+ with CGO enabled.

## Quick Start

```rust
use whatsmeow::{WhatsApp, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let client = WhatsApp::connect("session.db")
        .device_name("My Rust App")
        .on_qr(|qr| async move {
            if let Some(code) = qr.code() {
                println!("📱 Scan: {}", code);
            }
        })
        .on_message(|msg| async move {
            println!("📩 {}: {}", msg.sender_name(), msg.text());
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

## Stream-based Events

```rust
use futures::StreamExt;
use whatsmeow::Event;

let mut events = client.events();
while let Some(event) = events.next().await {
    match event {
        Event::Message(msg) => println!("{}: {}", msg.sender_name(), msg.text()),
        Event::Connected => println!("Connected!"),
        _ => {}
    }
}
```

## Project Structure

```
whatsmeow-rs/
├── crates/
│   ├── whatsmeow-sys/     # Raw FFI + Go source
│   └── whatsmeow/         # Safe Rust API
└── Cargo.toml
```

## Requirements

- **Rust** 2024 edition (1.85+)
- **Go** 1.21+ with CGO enabled
- **Windows**: MSVC toolchain
- **Linux/macOS**: GCC or Clang

## License

MIT
