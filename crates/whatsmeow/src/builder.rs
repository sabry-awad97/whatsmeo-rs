//! Fluent builder for WhatsApp client

use std::future::Future;
use std::path::Path;
use std::sync::Arc;

use crate::client::WhatsApp;
use crate::error::Result;
use crate::events::{MessageEvent, QrEvent};
use crate::ffi::FfiClient;
use crate::inner::InnerClient;

/// Builder for configuring a WhatsApp client
pub struct WhatsAppBuilder {
    db_path: String,
    device_name: String,
    inner: Option<Arc<InnerClient>>,
}

impl WhatsAppBuilder {
    pub(crate) fn new(db_path: impl AsRef<Path>) -> Self {
        Self {
            db_path: db_path.as_ref().to_string_lossy().into_owned(),
            device_name: "WhatsApp-RS".to_string(),
            inner: None,
        }
    }

    /// Set a custom device name (shown in WhatsApp's "Linked Devices" list)
    pub fn device_name(mut self, name: impl Into<String>) -> Self {
        self.device_name = name.into();
        self
    }

    fn ensure_inner(&mut self) -> Result<&Arc<InnerClient>> {
        if self.inner.is_none() {
            let ffi = FfiClient::new(&self.db_path, &self.device_name)?;
            self.inner = Some(Arc::new(InnerClient::new(ffi)));
        }
        Ok(self.inner.as_ref().unwrap())
    }

    /// Register an async QR code handler
    pub fn on_qr<F, Fut>(mut self, f: F) -> Self
    where
        F: Fn(QrEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        if let Ok(inner) = self.ensure_inner() {
            inner.handlers.register_qr(f);
        }
        self
    }

    /// Register an async message handler
    pub fn on_message<F, Fut>(mut self, f: F) -> Self
    where
        F: Fn(MessageEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        if let Ok(inner) = self.ensure_inner() {
            inner.handlers.register_message(f);
        }
        self
    }

    /// Register an async connected handler
    pub fn on_connected<F, Fut>(mut self, f: F) -> Self
    where
        F: Fn(()) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        if let Ok(inner) = self.ensure_inner() {
            inner.handlers.register_connected(f);
        }
        self
    }

    /// Register an async disconnected handler
    pub fn on_disconnected<F, Fut>(mut self, f: F) -> Self
    where
        F: Fn(()) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        if let Ok(inner) = self.ensure_inner() {
            inner.handlers.register_disconnected(f);
        }
        self
    }

    /// Build the client without starting event loop
    pub async fn build(mut self) -> Result<WhatsApp> {
        let inner = self.ensure_inner()?.clone();
        inner.connect().await?;
        Ok(WhatsApp::from_inner(inner))
    }

    /// Build and run the client
    pub async fn run(self) -> Result<()> {
        let client = self.build().await?;
        client.run().await
    }
}
