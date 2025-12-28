# whatsmeow-rs 🦀💬

Idiomatic, thread-safe Rust bindings for WhatsApp via [WhatsMeow](https://github.com/tulir/whatsmeow) Go bridge.

## Features

- ✨ **Fluent builder API** - No `Arc`, no channels in user code
- 📡 **Callback & Stream events** - Choose your style
- 🔒 **Thread-safe** - Share clients across tasks
- 📊 **Tracing integration** - Structured logging
- 🧠 **Multi-client support** - Manage multiple accounts
- 🏗️ **Automated build** - One command `task build` for Go & Rust

## Quick Start

```rust
use whatsmeow::WhatsApp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // simplified: DLL path is now handled automatically
    WhatsApp::connect("session.db")
        .on_qr(|qr| {
            if let Some(code) = qr.code() {
                println!("Scan QR: {}", code);
            }
        })
        .on_message(|msg| {
            println!("{}: {}", msg.sender_name(), msg.text);
        })
        .run()
        .await
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
        Event::Message(msg) => println!("{}", msg.text),
        Event::Disconnected => break,
        _ => {}
    }
}
```

## Requirements

- **Rust** 1.70+
- **Go** 1.24+ (CGO required)
- **Task** (go-task) for automation

## Building

The project uses `task` for unified build management.

```powershell
# 1. Build everything (Go DLL + .lib + Rust)
task build

# 2. Run the basic example
task run
```

## Project Structure

```
whatsmeow-rs/
├── crates/
│   ├── whatsmeow-sys/ # Raw FFI bindings
│   └── whatsmeow/     # Safe Rust API (Main Crate)
├── go/
│   ├── bridge/        # Go WhatsMeow wrapper
│   └── scripts/       # MSVC build scripts
├── examples/           # Multi and Stream examples
└── Taskfile.yml        # Build automation
```

## License

MIT
