//! Internal client state

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use tokio::sync::watch;

use crate::error::Result;
use crate::event_bus::EventBus;
use crate::events::RawEvent;
use crate::ffi::FfiClient;
use crate::handlers::Handlers;
use crate::stream::EventStream;

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

        loop {
            if *shutdown.borrow() {
                tracing::info!("Shutting down");
                break;
            }

            let data = { ffi.lock().poll_event()? };

            if let Some(bytes) = data {
                if let Ok(raw) = serde_json::from_slice::<RawEvent>(&bytes) {
                    if let Ok(event) = raw.into_event() {
                        tracing::debug!(?event, "Event received");
                        handlers.dispatch(&event);
                        bus.emit(event);
                    }
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
