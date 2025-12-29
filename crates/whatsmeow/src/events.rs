//! Event types for WhatsApp client

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// WhatsApp JID (Jabber ID) - identifies users, groups, and broadcasts
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Jid(String);

impl Jid {
    /// Create a JID from a raw string (e.g., "1234567890@s.whatsapp.net")
    pub fn new(jid: impl Into<String>) -> Self {
        Self(jid.into())
    }

    /// Create a user JID from a phone number (adds @s.whatsapp.net)
    pub fn user(phone: impl AsRef<str>) -> Self {
        let phone = phone.as_ref().trim_start_matches('+');
        Self(format!("{}@s.whatsapp.net", phone))
    }

    /// Create a group JID (adds @g.us)
    pub fn group(group_id: impl AsRef<str>) -> Self {
        Self(format!("{}@g.us", group_id.as_ref()))
    }

    /// Get the raw JID string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if this is a group JID
    pub fn is_group(&self) -> bool {
        self.0.ends_with("@g.us")
    }

    /// Check if this is a user JID
    pub fn is_user(&self) -> bool {
        self.0.ends_with("@s.whatsapp.net")
    }
}

impl fmt::Display for Jid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Jid {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for Jid {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<&String> for Jid {
    fn from(s: &String) -> Self {
        Self::new(s.clone())
    }
}

impl AsRef<str> for Jid {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Source of media content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaSource {
    /// Load from a local file path
    LocalPath { path: std::path::PathBuf },
    /// Download from a remote URL
    RemoteUrl { url: String },
    /// Base64 encoded data
    Base64 { data: String },
    /// Raw bytes (already loaded)
    Bytes { data: Vec<u8> },
}

impl MediaSource {
    /// Create from a local file path
    pub fn file(path: impl Into<std::path::PathBuf>) -> Self {
        MediaSource::LocalPath { path: path.into() }
    }

    /// Create from a URL
    pub fn url(url: impl Into<String>) -> Self {
        MediaSource::RemoteUrl { url: url.into() }
    }

    /// Create from base64 encoded string
    pub fn base64(data: impl Into<String>) -> Self {
        MediaSource::Base64 { data: data.into() }
    }

    /// Create from raw bytes
    pub fn bytes(data: Vec<u8>) -> Self {
        MediaSource::Bytes { data }
    }
}

impl From<Vec<u8>> for MediaSource {
    fn from(data: Vec<u8>) -> Self {
        MediaSource::Bytes { data }
    }
}

impl From<std::path::PathBuf> for MediaSource {
    fn from(path: std::path::PathBuf) -> Self {
        MediaSource::LocalPath { path }
    }
}

/// Error type for MediaSource conversion
#[derive(Debug, thiserror::Error)]
pub enum MediaSourceError {
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid base64: {0}")]
    Base64Error(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

impl MediaSource {
    /// Load file contents (for LocalPath variant)
    pub fn load(&self) -> Result<Vec<u8>, MediaSourceError> {
        match self {
            MediaSource::Bytes { data } => Ok(data.clone()),
            MediaSource::LocalPath { path } => Ok(std::fs::read(path)?),
            MediaSource::Base64 { data } => {
                use base64::Engine;
                base64::engine::general_purpose::STANDARD
                    .decode(data)
                    .map_err(|e| MediaSourceError::Base64Error(e.to_string()))
            }
            MediaSource::RemoteUrl { url } => Err(MediaSourceError::InvalidUrl(format!(
                "Cannot load remote URL directly: {}",
                url
            ))),
        }
    }

    /// Detect MIME type from file signature (magic bytes)
    pub fn detect_mime_from_signature(data: &[u8]) -> String {
        if data.len() >= 8 {
            // PNG signature
            if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
                return "image/png".to_string();
            }
            // JPEG signature
            if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
                return "image/jpeg".to_string();
            }
            // WebP signature
            if data.len() >= 12
                && data[0..4] == [0x52, 0x49, 0x46, 0x46]
                && data[8..12] == [0x57, 0x45, 0x42, 0x50]
            {
                return "image/webp".to_string();
            }
            // GIF signature
            if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
                return "image/gif".to_string();
            }
            // PDF signature
            if data.starts_with(b"%PDF") {
                return "application/pdf".to_string();
            }
            // ZIP signature (also used by DOCX, XLSX, etc.)
            if data.starts_with(&[0x50, 0x4B, 0x03, 0x04])
                || data.starts_with(&[0x50, 0x4B, 0x05, 0x06])
            {
                return "application/zip".to_string();
            }
            // MP4 signature
            if data.len() >= 12 && data[4..8] == [0x66, 0x74, 0x79, 0x70] {
                return "video/mp4".to_string();
            }
            // OGG signature (used for audio/video)
            if data.starts_with(b"OggS") {
                return "audio/ogg".to_string();
            }
            // MP3 signature
            if data.starts_with(&[0xFF, 0xFB])
                || data.starts_with(&[0xFF, 0xF3])
                || data.starts_with(&[0xFF, 0xF2])
            {
                return "audio/mpeg".to_string();
            }
            // WAV signature
            if data.starts_with(b"RIFF") && data.len() >= 12 && data[8..12] == *b"WAVE" {
                return "audio/wav".to_string();
            }
        }
        "application/octet-stream".to_string() // Default fallback
    }
}

/// Represents different types of outgoing WhatsApp messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// Plain text message
    Text(String),
    /// Image message
    Image {
        /// Image source (file, URL, base64, or raw bytes)
        source: MediaSource,
        /// MIME type (auto-detected if None)
        mime_type: Option<String>,
        /// Optional caption
        caption: Option<String>,
    },
    // Future: Video, Document, Audio, Location, Contact, etc.
}

impl MessageType {
    /// Create a text message
    pub fn text(s: impl Into<String>) -> Self {
        MessageType::Text(s.into())
    }

    /// Create an image message with explicit MIME type
    pub fn image(source: impl Into<MediaSource>, mime_type: impl Into<String>) -> Self {
        MessageType::Image {
            source: source.into(),
            mime_type: Some(mime_type.into()),
            caption: None,
        }
    }

    /// Create an image message with auto-detected MIME type
    pub fn image_auto(source: impl Into<MediaSource>) -> Self {
        MessageType::Image {
            source: source.into(),
            mime_type: None,
            caption: None,
        }
    }

    /// Create an image message with a caption and explicit MIME type
    pub fn image_with_caption(
        source: impl Into<MediaSource>,
        mime_type: impl Into<String>,
        caption: impl Into<String>,
    ) -> Self {
        MessageType::Image {
            source: source.into(),
            mime_type: Some(mime_type.into()),
            caption: Some(caption.into()),
        }
    }

    /// Create an image message with a caption and auto-detected MIME type
    pub fn image_auto_with_caption(
        source: impl Into<MediaSource>,
        caption: impl Into<String>,
    ) -> Self {
        MessageType::Image {
            source: source.into(),
            mime_type: None,
            caption: Some(caption.into()),
        }
    }

    /// Get text content if this is a text message
    pub fn as_text(&self) -> Option<&str> {
        match self {
            MessageType::Text(s) => Some(s),
            _ => None,
        }
    }
}

impl From<String> for MessageType {
    fn from(s: String) -> Self {
        MessageType::Text(s)
    }
}

impl From<&str> for MessageType {
    fn from(s: &str) -> Self {
        MessageType::Text(s.to_string())
    }
}

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

/// JID (WhatsApp ID) from Go JSON deserialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JidInfo {
    #[serde(rename = "User")]
    pub user: String,
    #[serde(rename = "Server")]
    pub server: String,
    #[serde(rename = "Device", default)]
    pub device: u16,
}

impl JidInfo {
    /// Convert to a Jid for sending
    pub fn to_jid(&self) -> Jid {
        Jid::new(format!("{}@{}", self.user, self.server))
    }
}

impl std::fmt::Display for JidInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.user, self.server)
    }
}

impl From<JidInfo> for Jid {
    fn from(info: JidInfo) -> Self {
        info.to_jid()
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
