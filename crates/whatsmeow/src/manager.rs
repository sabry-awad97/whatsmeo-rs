//! Multi-client management

use std::path::PathBuf;
use std::sync::Arc;

use dashmap::DashMap;

use crate::builder::WhatsAppBuilder;
use crate::client::WhatsApp;
use crate::error::{Error, Result};
use crate::inner::InnerClient;

/// Unique identifier for a client
pub type ClientId = String;

/// Manager for multiple WhatsApp client instances
pub struct WhatsAppManager {
    clients: DashMap<ClientId, Arc<InnerClient>>,
}

impl WhatsAppManager {
    /// Create a new manager
    pub fn new() -> Self {
        Self {
            clients: DashMap::new(),
        }
    }

    /// Spawn a new client with the given ID
    pub fn spawn(
        &self,
        id: impl Into<ClientId>,
        db_path: impl Into<PathBuf>,
    ) -> Result<WhatsAppBuilder> {
        let id = id.into();

        if self.clients.contains_key(&id) {
            return Err(Error::Init(format!("Client {} already exists", id)));
        }

        Ok(WhatsAppBuilder::new(db_path.into()))
    }

    /// Get an existing client by ID
    pub fn get(&self, id: &str) -> Option<WhatsApp> {
        self.clients
            .get(id)
            .map(|inner| WhatsApp::from_inner(inner.clone()))
    }

    /// Shutdown and remove a client
    pub fn shutdown(&self, id: &str) {
        if let Some((_, client)) = self.clients.remove(id) {
            client.disconnect();
            tracing::info!(client_id = %id, "Client shut down");
        }
    }

    /// Shutdown all clients
    pub fn shutdown_all(&self) {
        for entry in self.clients.iter() {
            entry.value().disconnect();
        }
        self.clients.clear();
        tracing::info!("All clients shut down");
    }

    /// Get number of active clients
    pub fn count(&self) -> usize {
        self.clients.len()
    }

    /// List all client IDs
    pub fn list(&self) -> Vec<ClientId> {
        self.clients.iter().map(|e| e.key().clone()).collect()
    }
}

impl Default for WhatsAppManager {
    fn default() -> Self {
        Self::new()
    }
}
