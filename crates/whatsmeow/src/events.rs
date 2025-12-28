//! Event types for WhatsApp client

use serde::{Deserialize, Serialize};

/// All events emitted by the WhatsApp client
#[derive(Debug, Clone)]
pub enum Event {
    /// QR code for authentication
    Qr(QrEvent),
    /// Pairing successful
    PairSuccess(PairSuccessEvent),
    /// Successfully connected
    Connected,
    /// Disconnected from WhatsApp
    Disconnected,
    /// Logged out
    LoggedOut(LoggedOutEvent),
    /// Incoming message
    Message(MessageEvent),
    /// Message delivery receipt
    Receipt(ReceiptEvent),
    /// Presence update
    Presence(PresenceEvent),
    /// History sync progress
    HistorySync,
    /// Unknown event type
    Unknown(String),
}

/// QR code event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrEvent {
    /// QR codes (multiple codes for retries)
    pub codes: Vec<String>,
}

impl QrEvent {
    /// Get the current/first QR code
    pub fn code(&self) -> Option<&str> {
        self.codes.first().map(|s| s.as_str())
    }
}

/// Pair success event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairSuccessEvent {
    pub id: String,
    pub business_name: String,
    pub platform: String,
}

/// Logged out event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggedOutEvent {
    pub on_connect: bool,
    pub reason: i32,
}

/// Incoming message event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEvent {
    pub id: String,
    pub from: String,
    pub chat: String,
    pub text: String,
    pub timestamp: i64,
    pub is_group: bool,
    #[serde(default)]
    pub push_name: String,
}

impl MessageEvent {
    pub fn is_group(&self) -> bool {
        self.is_group
    }

    pub fn sender_name(&self) -> &str {
        if !self.push_name.is_empty() {
            &self.push_name
        } else {
            self.from.split('@').next().unwrap_or(&self.from)
        }
    }
}

/// Message receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptEvent {
    pub message_ids: Vec<String>,
    pub chat: String,
    pub sender: String,
    #[serde(rename = "type")]
    pub receipt_type: String,
    pub timestamp: i64,
}

/// Presence event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceEvent {
    pub from: String,
    pub unavailable: bool,
    pub last_seen: i64,
}

impl PresenceEvent {
    pub fn is_online(&self) -> bool {
        !self.unavailable
    }
}

/// Raw event from FFI (internal)
#[derive(Debug, Deserialize)]
pub(crate) struct RawEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[allow(dead_code)]
    pub timestamp: i64,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

impl RawEvent {
    pub fn into_event(self) -> Result<Event, serde_json::Error> {
        match self.event_type.as_str() {
            "qr" => {
                if let Some(data) = self.data {
                    Ok(Event::Qr(serde_json::from_value(data)?))
                } else {
                    Ok(Event::Unknown("qr_no_data".into()))
                }
            }
            "pair_success" => {
                if let Some(data) = self.data {
                    Ok(Event::PairSuccess(serde_json::from_value(data)?))
                } else {
                    Ok(Event::Connected)
                }
            }
            "connected" => Ok(Event::Connected),
            "disconnected" => Ok(Event::Disconnected),
            "logged_out" => {
                if let Some(data) = self.data {
                    Ok(Event::LoggedOut(serde_json::from_value(data)?))
                } else {
                    Ok(Event::Disconnected)
                }
            }
            "message" => {
                if let Some(data) = self.data {
                    Ok(Event::Message(serde_json::from_value(data)?))
                } else {
                    Ok(Event::Unknown("message_no_data".into()))
                }
            }
            "receipt" => {
                if let Some(data) = self.data {
                    Ok(Event::Receipt(serde_json::from_value(data)?))
                } else {
                    Ok(Event::Unknown("receipt_no_data".into()))
                }
            }
            "presence" => {
                if let Some(data) = self.data {
                    Ok(Event::Presence(serde_json::from_value(data)?))
                } else {
                    Ok(Event::Unknown("presence_no_data".into()))
                }
            }
            "history_sync" => Ok(Event::HistorySync),
            other => Ok(Event::Unknown(other.to_string())),
        }
    }
}
