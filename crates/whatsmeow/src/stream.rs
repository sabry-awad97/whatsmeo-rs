//! Async Stream-based event access

use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Stream;
use tokio::sync::broadcast;

use crate::events::Event;

/// Async stream of WhatsApp events
pub struct EventStream {
    rx: broadcast::Receiver<Event>,
}

impl EventStream {
    pub(crate) fn new(rx: broadcast::Receiver<Event>) -> Self {
        Self { rx }
    }
}

impl Stream for EventStream {
    type Item = Event;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.rx.try_recv() {
            Ok(event) => Poll::Ready(Some(event)),
            Err(broadcast::error::TryRecvError::Empty) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(broadcast::error::TryRecvError::Lagged(_)) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(broadcast::error::TryRecvError::Closed) => Poll::Ready(None),
        }
    }
}

impl Clone for EventStream {
    fn clone(&self) -> Self {
        Self {
            rx: self.rx.resubscribe(),
        }
    }
}
