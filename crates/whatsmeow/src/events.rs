//! Event types for WhatsApp client

use serde::{Deserialize, Serialize};

/// All events emitted by the WhatsApp client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Event {
    /// QR code for authentication
    Qr(QrEvent),
    /// Successfully connected
    Connected,
    /// Disconnected from WhatsApp
    Disconnected,
    /// Incoming message
    Message(MessageEvent),
    /// Message delivery receipt
    Receipt(ReceiptEvent),
    /// Presence update
    Presence(PresenceEvent),
}

/// QR code event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrEvent {
    pub code: String,
    pub timeout_seconds: u32,
}

/// Incoming message event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEvent {
    pub id: String,
    pub from: String,
    pub to: String,
    pub text: String,
    pub timestamp: i64,
    pub is_group: bool,
}

impl MessageEvent {
    pub fn is_group(&self) -> bool {
        self.is_group
    }

    pub fn sender_name(&self) -> &str {
        self.from.split('@').next().unwrap_or(&self.from)
    }
}

/// Message receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptEvent {
    pub message_id: String,
    pub status: ReceiptStatus,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReceiptStatus {
    Sent,
    Delivered,
    Read,
}

/// Presence event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceEvent {
    pub jid: String,
    pub is_online: bool,
    pub last_seen: Option<i64>,
}

/// Raw event from FFI (internal)
#[derive(Debug, Deserialize)]
pub(crate) struct RawEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub timestamp: i64,
    pub data: serde_json::Value,
}

impl RawEvent {
    pub fn into_event(self) -> Result<Event, serde_json::Error> {
        match self.event_type.as_str() {
            "qr" => Ok(Event::Qr(serde_json::from_value(self.data)?)),
            "connected" => Ok(Event::Connected),
            "disconnected" => Ok(Event::Disconnected),
            "message" => Ok(Event::Message(serde_json::from_value(self.data)?)),
            "receipt" => Ok(Event::Receipt(serde_json::from_value(self.data)?)),
            "presence" => Ok(Event::Presence(serde_json::from_value(self.data)?)),
            _ => Ok(Event::Disconnected),
        }
    }
}
