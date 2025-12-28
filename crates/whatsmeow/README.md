# whatsmeow

Idiomatic, thread-safe Rust bindings for WhatsApp via [WhatsMeow](https://github.com/tulir/whatsmeow).

[![Crates.io](https://img.shields.io/crates/v/whatsmeow.svg)](https://crates.io/crates/whatsmeow)
[![Documentation](https://docs.rs/whatsmeow/badge.svg)](https://docs.rs/whatsmeow)

## Features

- ✨ **Fluent builder API** with chainable methods
- 📡 **Dual event model**: Callbacks or async streams
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
        .device_name("My App")  // Shows in WhatsApp
        .on_qr(|qr| {
            if let Some(code) = qr.code() {
                println!("Scan: {}", code);
            }
        })
        .on_message(|msg| {
            println!("{}: {}", msg.sender_name(), msg.text());
        })
        .build()
        .await?;

    // Graceful Ctrl+C handling
    tokio::select! {
        r = client.run() => r?,
        _ = tokio::signal::ctrl_c() => client.disconnect(),
    }
    Ok(())
}
```

## Memory Tracking

```rust
use whatsmeow::TrackedAllocator;

#[global_allocator]
static ALLOC: TrackedAllocator = TrackedAllocator::new();

// On shutdown:
ALLOC.print_stats();
```

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
