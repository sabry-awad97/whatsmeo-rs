# whatsmeow-rs 🦀💬

Idiomatic, thread-safe Rust bindings for WhatsApp via [WhatsMeow](https://github.com/tulir/whatsmeow) Go bridge.

[![Crates.io](https://img.shields.io/crates/v/whatsmeow.svg)](https://crates.io/crates/whatsmeow)
[![Documentation](https://docs.rs/whatsmeow/badge.svg)](https://docs.rs/whatsmeow)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Features

- ✨ **Fluent builder API** - No `Arc`, no channels in user code
- 📡 **Callback & Stream events** - Choose your style
- 🔒 **Thread-safe** - Share clients across tasks
- 📊 **Tracing integration** - Structured logging with FFI operation timing
- 🧠 **Multi-client support** - Manage multiple accounts
- 📦 **Automated Go bridge** - Compiles automatically via `build.rs`
- 🎯 **Memory tracking** - Built-in allocator for debugging FFI leaks
- 🏷️ **Custom device name** - Shows your app name in WhatsApp's "Linked Devices"

## Installation

```toml
[dependencies]
whatsmeow = "0.1.3"
tokio = { version = "1", features = ["full"] }
```

> **Note**: Requires Go 1.21+ with CGO enabled for the Go bridge compilation.

## Quick Start

```rust
use whatsmeow::{WhatsApp, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let client = WhatsApp::connect("storage/session.db")
        .device_name("My Rust App")  // Custom name in WhatsApp
        .on_qr(|qr| {
            if let Some(code) = qr.code() {
                println!("📱 Scan QR: {}", code);
            }
        })
        .on_connected(|_| println!("✅ Connected!"))
        .on_message(|msg| {
            let text = msg.text();
            if !text.is_empty() {
                println!("📩 {}: {}", msg.sender_name(), text);
            }
        })
        .build()
        .await?;

    // Graceful shutdown on Ctrl+C
    tokio::select! {
        result = client.run() => result?,
        _ = tokio::signal::ctrl_c() => {
            println!("👋 Shutting down...");
            client.disconnect();
        }
    }

    Ok(())
}
```

## Stream-based Events

```rust
use futures::StreamExt;
use whatsmeow::{Event, WhatsApp};

let client = WhatsApp::connect("session.db").build().await?;

let mut events = client.events();
while let Some(event) = events.next().await {
    match event {
        Event::Message(msg) => println!("{}: {}", msg.sender_name(), msg.text()),
        Event::Connected => println!("Connected!"),
        Event::Disconnected => break,
        _ => {}
    }
}
```

## Memory Tracking

Track FFI memory operations to detect leaks:

```rust
use whatsmeow::TrackedAllocator;

#[global_allocator]
static ALLOCATOR: TrackedAllocator = TrackedAllocator::new();

fn main() {
    // ... your code ...

    // Print stats on shutdown
    ALLOCATOR.print_stats();
}
```

## Building from Source

```bash
# Clone the repository
git clone https://github.com/sabry-awad97/whatsmeow-rs
cd whatsmeow-rs

# Run the example
cargo run --example basic
```

The Go bridge compiles automatically during `cargo build`. No manual steps needed!

## Project Structure

```
whatsmeow-rs/
├── crates/
│   ├── whatsmeow-sys/     # Raw FFI bindings + Go source
│   │   ├── go/bridge/     # Go WhatsMeow wrapper
│   │   └── src/           # Rust FFI declarations
│   └── whatsmeow/         # Safe, idiomatic Rust API
├── examples/              # Usage examples
└── Cargo.toml             # Workspace root
```

## Requirements

- **Rust** 2024 edition (1.85+)
- **Go** 1.21+ with CGO enabled
- **Windows**: MSVC toolchain (for `.lib` generation)
- **Linux/macOS**: GCC or Clang

## License

MIT
