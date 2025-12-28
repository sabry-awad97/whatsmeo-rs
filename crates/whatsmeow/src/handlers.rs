//! Callback-based event handling

use parking_lot::RwLock;
use std::sync::Arc;

use crate::events::{Event, MessageEvent, PresenceEvent, QrEvent, ReceiptEvent};

type Callback<T> = Arc<dyn Fn(T) + Send + Sync + 'static>;

/// Registry for event callbacks
pub(crate) struct Handlers {
    on_qr: RwLock<Vec<Callback<QrEvent>>>,
    on_message: RwLock<Vec<Callback<MessageEvent>>>,
    on_connected: RwLock<Vec<Callback<()>>>,
    on_disconnected: RwLock<Vec<Callback<()>>>,
    on_receipt: RwLock<Vec<Callback<ReceiptEvent>>>,
    on_presence: RwLock<Vec<Callback<PresenceEvent>>>,
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

    pub fn register_qr<F: Fn(QrEvent) + Send + Sync + 'static>(&self, f: F) {
        self.on_qr.write().push(Arc::new(f));
    }

    pub fn register_message<F: Fn(MessageEvent) + Send + Sync + 'static>(&self, f: F) {
        self.on_message.write().push(Arc::new(f));
    }

    pub fn register_connected<F: Fn(()) + Send + Sync + 'static>(&self, f: F) {
        self.on_connected.write().push(Arc::new(f));
    }

    pub fn register_disconnected<F: Fn(()) + Send + Sync + 'static>(&self, f: F) {
        self.on_disconnected.write().push(Arc::new(f));
    }

    pub fn dispatch(&self, event: &Event) {
        match event {
            Event::Qr(data) => {
                for h in self.on_qr.read().iter() {
                    h(data.clone());
                }
            }
            Event::Message(data) => {
                for h in self.on_message.read().iter() {
                    h(data.clone());
                }
            }
            Event::Connected | Event::PairSuccess(_) => {
                for h in self.on_connected.read().iter() {
                    h(());
                }
            }
            Event::Disconnected | Event::LoggedOut(_) => {
                for h in self.on_disconnected.read().iter() {
                    h(());
                }
            }
            Event::Receipt(data) => {
                for h in self.on_receipt.read().iter() {
                    h(data.clone());
                }
            }
            Event::Presence(data) => {
                for h in self.on_presence.read().iter() {
                    h(data.clone());
                }
            }
            // Ignored events
            Event::HistorySync | Event::Unknown(_) => {}
        }
    }
}

impl Default for Handlers {
    fn default() -> Self {
        Self::new()
    }
}
