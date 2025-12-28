//! Public WhatsApp client interface

use std::path::Path;
use std::sync::Arc;

use crate::builder::WhatsAppBuilder;
use crate::error::Result;
use crate::inner::InnerClient;
use crate::stream::EventStream;

/// WhatsApp client for sending and receiving messages
#[derive(Clone)]
pub struct WhatsApp {
    pub(crate) inner: Arc<InnerClient>,
}

impl WhatsApp {
    /// Start building a new WhatsApp client
    pub fn connect(_dll_path: impl AsRef<Path>, db_path: impl AsRef<Path>) -> WhatsAppBuilder {
        WhatsAppBuilder::new(db_path)
    }

    pub(crate) fn from_inner(inner: Arc<InnerClient>) -> Self {
        Self { inner }
    }

    /// Get an async stream of events
    pub fn events(&self) -> EventStream {
        self.inner.events()
    }

    /// Run the client event loop
    pub async fn run(&self) -> Result<()> {
        self.inner.run().await
    }

    /// Send a text message
    pub fn send(&self, to: &str, text: &str) -> Result<()> {
        self.inner.send_message(to, text)
    }

    /// Disconnect from WhatsApp
    pub fn disconnect(&self) {
        self.inner.disconnect();
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }
}
