//! # WhatsApp Rust Client
//!
//! Idiomatic Rust bindings for WhatsApp messaging via WhatsMeow.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use whatsmeow::WhatsApp;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     WhatsApp::connect("whatsapp.dll", "session.db")
//!         .on_qr(|qr| println!("Scan: {}", qr.code))
//!         .on_message(|msg| println!("{}: {}", msg.from, msg.text))
//!         .run()
//!         .await?;
//!
//!     Ok(())
//! }
//! ```

mod builder;
mod client;
mod error;
mod event_bus;
mod events;
mod ffi;
mod handlers;
mod inner;
mod stream;

pub use builder::WhatsAppBuilder;
pub use client::WhatsApp;
pub use error::{Error, Result};
pub use events::{Event, MessageEvent, PresenceEvent, QrEvent, ReceiptEvent, ReceiptStatus};
pub use stream::EventStream;

/// Initialize default tracing subscriber
pub fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("whatsmeow=info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().compact())
        .init();
}
