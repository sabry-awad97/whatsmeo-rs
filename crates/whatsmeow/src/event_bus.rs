//! Internal event bus

use tokio::sync::broadcast;

use crate::events::Event;
use crate::stream::EventStream;

const EVENT_CHANNEL_CAPACITY: usize = 256;

pub(crate) struct EventBus {
    tx: broadcast::Sender<Event>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        Self { tx }
    }

    pub fn emit(&self, event: Event) {
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> EventStream {
        EventStream::new(self.tx.subscribe())
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}
