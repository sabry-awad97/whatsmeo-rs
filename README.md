# whatsmeow-rs 🦀💬

Idiomatic Rust bindings for WhatsApp via [WhatsMeow](https://github.com/tulir/whatsmeow).

## Features

- ✨ **Fluent builder API** - No `Arc`, no channels in user code
- 📡 **Callback & Stream events** - Choose your style
- 🔒 **Thread-safe** - Share clients across tasks
- 📊 **Tracing integration** - Structured logging
- 🧠 **Multi-client support** - Manage multiple accounts

## Quick Start

```rust
use whatsmeow::WhatsApp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    WhatsApp::connect("whatsapp.dll", "session.db")
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

let client = WhatsApp::connect("dll", "db").build().await?;

let mut events = client.events();
while let Some(event) = events.next().await {
    match event {
        Event::Message(msg) => println!("{}", msg.text),
        Event::Disconnected => break,
        _ => {}
    }
}
```

## Building

### Prerequisites

- Rust 1.70+
- Go 1.21+
- CGO enabled (for building DLL)

### Build Go DLL

```powershell
cd go/bridge
$env:CGO_ENABLED="1"
go build -buildmode=c-shared -o ../target/whatsmeow.dll .
```

### Build Rust

```bash
cargo build --workspace
```

## Project Structure

```
whatsmeow-rs/
├── crates/
│   ├── whatsmeow-sys/    # Raw FFI bindings
│   └── whatsmeow/        # Safe Rust API
├── go/bridge/            # Go WhatsMeow wrapper
└── examples/             # Usage examples
```

## License

MIT
