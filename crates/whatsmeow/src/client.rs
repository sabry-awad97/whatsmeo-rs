//! Public WhatsApp client interface

use std::path::Path;
use std::sync::Arc;

use crate::builder::WhatsAppBuilder;
use crate::error::Result;
use crate::events::{Jid, MessageType};
use crate::inner::InnerClient;
use crate::stream::EventStream;

/// WhatsApp client for sending and receiving messages
#[derive(Clone)]
pub struct WhatsApp {
    pub(crate) inner: Arc<InnerClient>,
}

impl WhatsApp {
    /// Start building a new WhatsApp client
    pub fn connect(db_path: impl AsRef<Path>) -> WhatsAppBuilder {
        WhatsAppBuilder::new(db_path)
    }

    pub(crate) fn from_inner(inner: Arc<InnerClient>) -> Self {
        Self { inner }
    }

    /// Get an async stream of events
    pub fn events(&self) -> EventStream {
        self.inner.events()
    }

    /// Run the client event loop
    pub async fn run(&self) -> Result<()> {
        self.inner.run().await
    }

    /// Send a message to a JID
    ///
    /// # Examples
    /// ```rust,no_run
    /// use whatsmeow::{Jid, MessageType};
    ///
    /// // Send with string (auto-converted)
    /// client.send("1234567890@s.whatsapp.net", "Hello!")?;
    ///
    /// // Send with Jid builder
    /// client.send(Jid::user("+1234567890"), "Hello!")?;
    ///
    /// // Send to a group
    /// client.send(Jid::group("123456789"), MessageType::Text("Hello group!".into()))?;
    ///
    /// // Send an image from file path
    /// client.send(Jid::user("1234567890"), MessageType::image(MediaSource::file("photo.jpg"), "image/jpeg"))?;
    ///
    /// // Send an image from bytes
    /// let data = std::fs::read("photo.jpg")?;
    /// client.send(Jid::user("1234567890"), MessageType::image(data, "image/jpeg"))?;
    /// ```
    pub fn send(&self, to: impl Into<Jid>, message: impl Into<MessageType>) -> Result<()> {
        let jid: Jid = to.into();
        let msg: MessageType = message.into();

        match msg {
            MessageType::Text(text) => self.inner.send_message(jid.as_str(), &text),
            MessageType::Image {
                source,
                mime_type,
                caption,
            } => {
                // Resolve the media source to bytes
                let data = match source.load() {
                    Ok(data) => data,
                    Err(e) => {
                        return Err(crate::error::Error::Send(format!(
                            "Failed to load media: {}",
                            e
                        )));
                    }
                };

                // Auto-detect MIME type from file signature if not provided
                let detected_mime = mime_type.unwrap_or_else(|| {
                    crate::events::MediaSource::detect_mime_from_signature(&data)
                });

                self.inner
                    .send_image(jid.as_str(), &data, &detected_mime, caption.as_deref())
            }
        }
    }

    /// Disconnect from WhatsApp
    pub fn disconnect(&self) {
        self.inner.disconnect();
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }
}
