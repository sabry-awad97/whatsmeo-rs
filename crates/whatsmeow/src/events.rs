//! Event types for WhatsApp client

use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    /// Offline sync preview
    OfflineSyncPreview(OfflineSyncPreviewEvent),
    /// Offline sync completed
    OfflineSyncCompleted(OfflineSyncCompletedEvent),
    /// Unknown event type (contains raw JSON for inspection)
    Unknown {
        event_type: String,
        data: Option<Value>,
    },
}

/// QR code event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrEvent {
    /// QR codes (multiple codes for retries)
    #[serde(rename = "Codes")]
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
    #[serde(rename = "ID")]
    pub id: Jid,
    #[serde(rename = "BusinessName")]
    pub business_name: String,
    #[serde(rename = "Platform")]
    pub platform: String,
}

/// JID (WhatsApp ID)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jid {
    #[serde(rename = "User")]
    pub user: String,
    #[serde(rename = "Server")]
    pub server: String,
    #[serde(rename = "Device", default)]
    pub device: u16,
}

impl std::fmt::Display for Jid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.user, self.server)
    }
}

/// Logged out event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggedOutEvent {
    #[serde(rename = "OnConnect")]
    pub on_connect: bool,
    #[serde(rename = "Reason")]
    pub reason: i32,
}

/// Message info from WhatsApp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageInfo {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Chat")]
    pub chat: String,
    #[serde(rename = "Sender")]
    pub sender: String,
    #[serde(rename = "SenderAlt", default)]
    pub sender_alt: String,
    #[serde(rename = "IsFromMe")]
    pub is_from_me: bool,
    #[serde(rename = "IsGroup")]
    pub is_group: bool,
    #[serde(rename = "PushName", default)]
    pub push_name: String,
    #[serde(rename = "Timestamp")]
    pub timestamp: String,
    #[serde(rename = "Type", default)]
    pub message_type: String,
    #[serde(rename = "MediaType", default)]
    pub media_type: String,
    #[serde(rename = "Category", default)]
    pub category: String,
}

/// Incoming message event (full structure from Go)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEvent {
    #[serde(rename = "Info")]
    pub info: MessageInfo,
    #[serde(rename = "Message", default)]
    pub message: Option<Value>,
    #[serde(rename = "IsEdit", default)]
    pub is_edit: bool,
    #[serde(rename = "IsEphemeral", default)]
    pub is_ephemeral: bool,
    #[serde(rename = "IsViewOnce", default)]
    pub is_view_once: bool,
    #[serde(rename = "IsDocumentWithCaption", default)]
    pub is_document_with_caption: bool,
}

impl MessageEvent {
    pub fn is_group(&self) -> bool {
        self.info.is_group
    }

    pub fn sender_name(&self) -> &str {
        if !self.info.push_name.is_empty() {
            &self.info.push_name
        } else {
            self.info
                .sender
                .split('@')
                .next()
                .unwrap_or(&self.info.sender)
        }
    }

    /// Extract text from the message (handles conversation + extended text)
    pub fn text(&self) -> String {
        if let Some(msg) = &self.message {
            // Try conversation first
            if let Some(text) = msg.get("conversation").and_then(|v| v.as_str()) {
                return text.to_string();
            }
            // Try extended text message
            if let Some(ext) = msg.get("extendedTextMessage")
                && let Some(text) = ext.get("text").and_then(|v| v.as_str())
            {
                return text.to_string();
            }
        }
        String::new()
    }
}

/// Message receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptEvent {
    #[serde(rename = "MessageIDs")]
    pub message_ids: Vec<String>,
    #[serde(rename = "Chat")]
    pub chat: String,
    #[serde(rename = "Sender")]
    pub sender: String,
    #[serde(rename = "Type")]
    pub receipt_type: String,
    #[serde(rename = "Timestamp")]
    pub timestamp: String,
}

/// Presence event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceEvent {
    #[serde(rename = "From")]
    pub from: String,
    #[serde(rename = "Unavailable")]
    pub unavailable: bool,
    #[serde(rename = "LastSeen")]
    pub last_seen: String,
}

impl PresenceEvent {
    pub fn is_online(&self) -> bool {
        !self.unavailable
    }
}

/// Offline sync preview event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineSyncPreviewEvent {
    #[serde(rename = "Total")]
    pub total: i32,
    #[serde(rename = "AppDataChanges")]
    pub app_data_changes: i32,
    #[serde(rename = "Messages")]
    pub messages: i32,
    #[serde(rename = "Notifications")]
    pub notifications: i32,
    #[serde(rename = "Receipts")]
    pub receipts: i32,
}

/// Offline sync completed event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineSyncCompletedEvent {
    #[serde(rename = "Count")]
    pub count: i32,
}

/// Raw event from FFI (internal)
#[derive(Debug, Deserialize)]
pub(crate) struct RawEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[allow(dead_code)]
    pub timestamp: i64,
    #[serde(default)]
    pub data: Option<Value>,
}

impl RawEvent {
    pub fn into_event(self) -> Result<Event, serde_json::Error> {
        match self.event_type.as_str() {
            "qr" => {
                if let Some(data) = self.data {
                    Ok(Event::Qr(serde_json::from_value(data)?))
                } else {
                    Ok(Event::Unknown {
                        event_type: "qr".into(),
                        data: None,
                    })
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
                    Ok(Event::Unknown {
                        event_type: "message".into(),
                        data: None,
                    })
                }
            }
            "receipt" => {
                if let Some(data) = self.data {
                    Ok(Event::Receipt(serde_json::from_value(data)?))
                } else {
                    Ok(Event::Unknown {
                        event_type: "receipt".into(),
                        data: None,
                    })
                }
            }
            "presence" => {
                if let Some(data) = self.data {
                    Ok(Event::Presence(serde_json::from_value(data)?))
                } else {
                    Ok(Event::Unknown {
                        event_type: "presence".into(),
                        data: None,
                    })
                }
            }
            "history_sync" => Ok(Event::HistorySync),
            "offline_sync_preview" => {
                if let Some(data) = self.data {
                    Ok(Event::OfflineSyncPreview(serde_json::from_value(data)?))
                } else {
                    Ok(Event::Unknown {
                        event_type: "offline_sync_preview".into(),
                        data: None,
                    })
                }
            }
            "offline_sync_completed" => {
                if let Some(data) = self.data {
                    Ok(Event::OfflineSyncCompleted(serde_json::from_value(data)?))
                } else {
                    Ok(Event::Unknown {
                        event_type: "offline_sync_completed".into(),
                        data: None,
                    })
                }
            }
            other => Ok(Event::Unknown {
                event_type: other.to_string(),
                data: self.data,
            }),
        }
    }
}
