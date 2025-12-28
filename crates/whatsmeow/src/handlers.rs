//! Callback-based event handling with async support

use parking_lot::RwLock;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::events::{Event, MessageEvent, PresenceEvent, QrEvent, ReceiptEvent};

/// Boxed future type for async callbacks
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Async callback type
type AsyncCallback<T> = Arc<dyn Fn(T) -> BoxFuture<'static, ()> + Send + Sync + 'static>;

/// Registry for event callbacks (supports async)
pub(crate) struct Handlers {
    on_qr: RwLock<Vec<AsyncCallback<QrEvent>>>,
    on_message: RwLock<Vec<AsyncCallback<MessageEvent>>>,
    on_connected: RwLock<Vec<AsyncCallback<()>>>,
    on_disconnected: RwLock<Vec<AsyncCallback<()>>>,
    on_receipt: RwLock<Vec<AsyncCallback<ReceiptEvent>>>,
    on_presence: RwLock<Vec<AsyncCallback<PresenceEvent>>>,
}

impl Handlers {
    pub fn new() -> Self {
        Self {
            on_qr: RwLock::new(Vec::new()),
            on_message: RwLock::new(Vec::new()),
            on_connected: RwLock::new(Vec::new()),
            on_disconnected: RwLock::new(Vec::new()),
            on_receipt: RwLock::new(Vec::new()),
            on_presence: RwLock::new(Vec::new()),
        }
    }

    pub fn register_qr<F, Fut>(&self, f: F)
    where
        F: Fn(QrEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.on_qr.write().push(Arc::new(move |e| Box::pin(f(e))));
    }

    pub fn register_message<F, Fut>(&self, f: F)
    where
        F: Fn(MessageEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.on_message
            .write()
            .push(Arc::new(move |e| Box::pin(f(e))));
    }

    pub fn register_connected<F, Fut>(&self, f: F)
    where
        F: Fn(()) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.on_connected
            .write()
            .push(Arc::new(move |e| Box::pin(f(e))));
    }

    pub fn register_disconnected<F, Fut>(&self, f: F)
    where
        F: Fn(()) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.on_disconnected
            .write()
            .push(Arc::new(move |e| Box::pin(f(e))));
    }

    /// Dispatch event to all registered handlers (spawns tasks for async execution)
    pub fn dispatch(&self, event: &Event) {
        match event {
            Event::Qr(data) => {
                let handlers = self.on_qr.read().clone();
                let data = data.clone();
                for h in handlers {
                    let data = data.clone();
                    tokio::spawn(async move { h(data).await });
                }
            }
            Event::Message(data) => {
                let handlers = self.on_message.read().clone();
                let data = data.clone();
                for h in handlers {
                    let data = data.clone();
                    tokio::spawn(async move { h(data).await });
                }
            }
            Event::Connected | Event::PairSuccess(_) => {
                let handlers = self.on_connected.read().clone();
                for h in handlers {
                    tokio::spawn(async move { h(()).await });
                }
            }
            Event::Disconnected | Event::LoggedOut(_) => {
                let handlers = self.on_disconnected.read().clone();
                for h in handlers {
                    tokio::spawn(async move { h(()).await });
                }
            }
            Event::Receipt(data) => {
                let handlers = self.on_receipt.read().clone();
                let data = data.clone();
                for h in handlers {
                    let data = data.clone();
                    tokio::spawn(async move { h(data).await });
                }
            }
            Event::Presence(data) => {
                let handlers = self.on_presence.read().clone();
                let data = data.clone();
                for h in handlers {
                    let data = data.clone();
                    tokio::spawn(async move { h(data).await });
                }
            }
            // Ignored events
            Event::HistorySync
            | Event::OfflineSyncPreview(_)
            | Event::OfflineSyncCompleted(_)
            | Event::Unknown { .. } => {}
        }
    }
}

impl Default for Handlers {
    fn default() -> Self {
        Self::new()
    }
}
