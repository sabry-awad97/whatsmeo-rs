//! Internal client state

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use parking_lot::Mutex;
use tokio::sync::watch;

use crate::error::Result;
use crate::event_bus::EventBus;
use crate::events::RawEvent;
use crate::ffi::FfiClient;
use crate::handlers::Handlers;
use crate::stream::EventStream;

/// Set to true to save one sample of each raw event type to debug_events/
const DEBUG_SAVE_EVENTS: bool = false;

pub(crate) struct InnerClient {
    pub ffi: Arc<Mutex<FfiClient>>,
    pub event_bus: EventBus,
    pub handlers: Arc<Handlers>,
    shutdown_tx: watch::Sender<bool>,
    shutdown_rx: watch::Receiver<bool>,
    connected: AtomicBool,
}

impl InnerClient {
    pub fn new(ffi: FfiClient) -> Self {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        Self {
            ffi: Arc::new(Mutex::new(ffi)),
            event_bus: EventBus::new(),
            handlers: Arc::new(Handlers::new()),
            shutdown_tx,
            shutdown_rx,
            connected: AtomicBool::new(false),
        }
    }

    #[tracing::instrument(skip(self), name = "whatsapp.connect")]
    pub async fn connect(&self) -> Result<()> {
        tracing::info!("Connecting to WhatsApp");
        self.ffi.lock().connect()?;
        self.connected.store(true, Ordering::SeqCst);
        tracing::info!("Connected to WhatsApp");
        Ok(())
    }

    pub async fn run(self: &Arc<Self>) -> Result<()> {
        tracing::info!("Starting event loop");

        let ffi = self.ffi.clone();
        let bus = self.event_bus.clone();
        let handlers = self.handlers.clone();
        let mut shutdown = self.shutdown_rx.clone();

        // Track which event types we've already saved (for debugging)
        let mut saved_event_types = std::collections::HashSet::new();
        let debug_dir = std::path::Path::new("debug_events");

        loop {
            if *shutdown.borrow() {
                tracing::info!("Shutting down");
                break;
            }

            let data = { ffi.lock().poll_event()? };

            if let Some(bytes) = data {
                // Save raw event for debugging (once per event type)
                if DEBUG_SAVE_EVENTS
                    && let Ok(raw) = serde_json::from_slice::<serde_json::Value>(&bytes)
                    && let Some(event_type) = raw.get("type").and_then(|t| t.as_str())
                    && !saved_event_types.contains(event_type)
                {
                    saved_event_types.insert(event_type.to_string());
                    let _ = std::fs::create_dir_all(debug_dir);
                    let filename = debug_dir.join(format!("{}.json", event_type));
                    if let Ok(pretty) = serde_json::to_string_pretty(&raw) {
                        let _ = std::fs::write(&filename, pretty);
                        tracing::info!("Saved raw event sample: {}", filename.display());
                    }
                }

                if let Ok(raw) = serde_json::from_slice::<RawEvent>(&bytes)
                    && let Ok(event) = raw.into_event()
                {
                    tracing::debug!(?event, "Event received");
                    handlers.dispatch(&event);
                    bus.emit(event);
                }
            } else {
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_millis(10)) => {}
                    _ = shutdown.changed() => break,
                }
            }
        }

        Ok(())
    }

    pub fn events(&self) -> EventStream {
        self.event_bus.subscribe()
    }

    pub fn send_message(&self, jid: &str, text: &str) -> Result<()> {
        self.ffi.lock().send_message(jid, text)
    }

    pub fn send_image(
        &self,
        jid: &str,
        data: &[u8],
        mime_type: &str,
        caption: Option<&str>,
    ) -> Result<()> {
        self.ffi.lock().send_image(jid, data, mime_type, caption)
    }

    pub fn disconnect(&self) {
        let _ = self.shutdown_tx.send(true);
        if let Some(client) = self.ffi.try_lock() {
            let _ = client.disconnect();
        }
        self.connected.store(false, Ordering::SeqCst);
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }
}

impl Drop for InnerClient {
    fn drop(&mut self) {
        let _ = self.shutdown_tx.send(true);
    }
}
