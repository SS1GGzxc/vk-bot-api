use crate::api::{LongPollServer, VkApi};
use crate::error::{VkError, VkResult};
use crate::handler::MessageHandler;
use crate::models::{Event, Update};
use futures::stream::{self, StreamExt};
use log::{error, info, warn};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Bot configuration
#[derive(Debug, Clone)]
pub struct VkBotConfig {
    /// Long Poll wait timeout in seconds (default: 25)
    pub long_poll_timeout: u64,
    /// Maximum number of concurrent handlers (default: 10)
    pub max_concurrent_handlers: usize,
    /// Enable automatic reconnection (default: true)
    pub auto_reconnect: bool,
    /// Reconnection delay in seconds (default: 5)
    pub reconnect_delay: u64,
    /// Maximum reconnection attempts (default: 10)
    pub max_reconnect_attempts: u32,
    /// Log updates (default: false)
    pub log_updates: bool,
}

impl Default for VkBotConfig {
    fn default() -> Self {
        Self {
            long_poll_timeout: 25,
            max_concurrent_handlers: 10,
            auto_reconnect: true,
            reconnect_delay: 5,
            max_reconnect_attempts: 10,
            log_updates: false,
        }
    }
}

/// Bot builder
#[derive(Debug, Default)]
pub struct VkBotBuilder {
    token: Option<String>,
    group_id: Option<i64>,
    config: VkBotConfig,
}

impl VkBotBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set access token
    pub fn token<T: Into<String>>(mut self, token: T) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Set group ID
    pub fn group_id(mut self, group_id: i64) -> Self {
        self.group_id = Some(group_id);
        self
    }

    /// Set Long Poll timeout
    pub fn long_poll_timeout(mut self, timeout: u64) -> Self {
        self.config.long_poll_timeout = timeout;
        self
    }

    /// Set max concurrent handlers
    pub fn max_concurrent_handlers(mut self, max: usize) -> Self {
        self.config.max_concurrent_handlers = max;
        self
    }

    /// Enable/disable auto reconnection
    pub fn auto_reconnect(mut self, enable: bool) -> Self {
        self.config.auto_reconnect = enable;
        self
    }

    /// Set reconnection delay
    pub fn reconnect_delay(mut self, delay: u64) -> Self {
        self.config.reconnect_delay = delay;
        self
    }

    /// Set max reconnection attempts
    pub fn max_reconnect_attempts(mut self, attempts: u32) -> Self {
        self.config.max_reconnect_attempts = attempts;
        self
    }

    /// Enable/disable update logging
    pub fn log_updates(mut self, enable: bool) -> Self {
        self.config.log_updates = enable;
        self
    }

    /// Build bot instance
    pub fn build(self) -> VkResult<VkBot> {
        let token = self
            .token
            .ok_or_else(|| VkError::ConfigError("Token is required".to_string()))?;
        let group_id = self
            .group_id
            .ok_or_else(|| VkError::ConfigError("Group ID is required".to_string()))?;

        let api = VkApi::new(&token)?;

        Ok(VkBot {
            api: Arc::new(api),
            group_id,
            config: Arc::new(self.config),
            long_poll_server: None,
            handlers: Arc::new(Vec::new()),
        })
    }
}

/// Main bot structure
pub struct VkBot {
    api: Arc<VkApi>,
    group_id: i64,
    config: Arc<VkBotConfig>,
    long_poll_server: Option<LongPollServer>,
    handlers: Arc<Vec<Box<dyn MessageHandler>>>,
}

use std::fmt;
impl fmt::Debug for VkBot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VkBot")
            .field("api", &self.api)
            .field("group_id", &self.group_id)
            .field("config", &self.config)
            .field("long_poll_server", &self.long_poll_server)
            .field(
                "handlers",
                &format_args!("Arc<Vec<Box<dyn MessageHandler>>>"),
            )
            .finish()
    }
}

impl VkBot {
    /// Create new bot with default config
    pub fn new<T: Into<String>>(token: T, group_id: i64) -> VkResult<Self> {
        VkBotBuilder::new().token(token).group_id(group_id).build()
    }

    /// Create new bot with builder
    pub fn builder() -> VkBotBuilder {
        VkBotBuilder::new()
    }

    /// Add message handler
    pub fn add_handler<H>(&mut self, handler: H)
    where
        H: MessageHandler + 'static,
    {
        let handlers =
            Arc::get_mut(&mut self.handlers).expect("Cannot add handler while bot is running");
        handlers.push(Box::new(handler));
    }

    /// Initialize bot (get Long Poll server)
    pub async fn init(&mut self) -> VkResult<()> {
        info!("Initializing bot for group {}...", self.group_id);

        self.long_poll_server = Some(self.api.groups_get_long_poll_server(self.group_id).await?);

        info!("Long Poll server obtained");

        // Get group info
        self.get_group_info().await?;

        Ok(())
    }

    /// Get group information
    async fn get_group_info(&self) -> VkResult<()> {
        let response = self
            .api
            .groups_get_by_id(&[self.group_id], Some("name,description,members_count"))
            .await?;

        if let Some(group) = response["response"].as_array().and_then(|arr| arr.first()) {
            let name = group["name"].as_str().unwrap_or("Unknown");
            let members = group["members_count"].as_i64().unwrap_or(0);
            info!("Group: {} (members: {})", name, members);
        }

        Ok(())
    }

    /// Perform Long Poll request
    async fn long_poll(&mut self) -> VkResult<Vec<Update>> {
        let server = self
            .long_poll_server
            .as_ref()
            .ok_or_else(|| VkError::ConfigError("Long Poll server not initialized".to_string()))?;

        let url = format!(
            "{}?act=a_check&key={}&ts={}&wait={}",
            server.server, server.key, server.ts, self.config.long_poll_timeout
        );

        let response = reqwest::get(&url).await?;

        if !response.status().is_success() {
            return Err(VkError::NetworkError(format!(
                "Long Poll HTTP error: {}",
                response.status()
            )));
        }

        let json_response: serde_json::Value = response.json().await?;

        // Handle Long Poll errors
        if let Some(failed) = json_response["failed"].as_i64() {
            match failed {
                1 => {
                    // Update ts only
                    if let Some(ts) = json_response["ts"].as_str() {
                        if let Some(ref mut server) = self.long_poll_server {
                            server.ts = ts.to_string();
                        }
                    }
                    return Ok(Vec::new());
                }
                2 | 3 => {
                    // Need new server
                    warn!("Long Poll failed {}, getting new server...", failed);
                    self.long_poll_server =
                        Some(self.api.groups_get_long_poll_server(self.group_id).await?);
                    return Ok(Vec::new());
                }
                _ => {
                    return Err(VkError::api_error(
                        failed as i32,
                        format!("Long Poll failed with code: {}", failed),
                    ));
                }
            }
        }

        // Update ts
        if let Some(ts) = json_response["ts"].as_str() {
            if let Some(ref mut server) = self.long_poll_server {
                server.ts = ts.to_string();
            }
        }

        // Parse updates
        let updates = if let Some(updates_array) = json_response["updates"].as_array() {
            updates_array
                .iter()
                .filter_map(|update| serde_json::from_value(update.clone()).ok())
                .collect()
        } else {
            Vec::new()
        };

        if self.config.log_updates && !updates.is_empty() {
            info!("Received {} updates", updates.len());
        }

        Ok(updates)
    }

    /// Handle updates
    async fn handle_updates(&self, updates: &[Update]) {
        let handlers = self.handlers.clone();
        let api = self.api.clone();

        // Process updates concurrently with limit
        let futures = updates.iter().map(|update| {
            let handlers = handlers.clone();
            let api = api.clone();
            async move {
                let event = Event::from_update(update);

                // Process with all handlers
                for handler in handlers.iter() {
                    if let Err(e) = handler.handle(&event, &api).await {
                        error!("Handler error: {}", e);
                    }
                }
            }
        });

        stream::iter(futures)
            .buffer_unordered(self.config.max_concurrent_handlers)
            .for_each(|_| async {})
            .await;
    }

    /// Main bot loop
    pub async fn run(&mut self) -> VkResult<()> {
        self.init().await?;

        info!("Bot started. Waiting for events...");

        let mut reconnect_attempts = 0;

        loop {
            match self.long_poll().await {
                Ok(updates) => {
                    reconnect_attempts = 0; // Reset on success

                    if !updates.is_empty() {
                        self.handle_updates(&updates).await;
                    }
                }
                Err(e) => {
                    error!("Long Poll error: {}", e);

                    if !self.config.auto_reconnect {
                        return Err(e);
                    }

                    reconnect_attempts += 1;

                    if reconnect_attempts > self.config.max_reconnect_attempts {
                        error!("Maximum reconnection attempts reached");
                        return Err(e);
                    }

                    warn!(
                        "Reconnecting in {} seconds... (attempt {}/{})",
                        self.config.reconnect_delay,
                        reconnect_attempts,
                        self.config.max_reconnect_attempts
                    );

                    sleep(Duration::from_secs(self.config.reconnect_delay)).await;

                    // Try to reinitialize
                    if let Err(e) = self.init().await {
                        error!("Failed to reinitialize: {}", e);
                    } else {
                        info!("Reconnected successfully");
                    }
                }
            }
        }
    }

    /// Get API client reference
    pub fn api(&self) -> &VkApi {
        &self.api
    }

    /// Get group ID
    pub fn group_id(&self) -> i64 {
        self.group_id
    }

    /// Get bot config
    pub fn config(&self) -> &VkBotConfig {
        &self.config
    }

    /// Send broadcast message to multiple users
    pub async fn broadcast(
        &self,
        message: &str,
        user_ids: &[i64],
        delay_ms: u64,
    ) -> VkResult<Vec<(i64, VkResult<i64>)>> {
        let mut results = Vec::new();

        for &user_id in user_ids {
            sleep(Duration::from_millis(delay_ms)).await;

            let result = self
                .api
                .messages_send(
                    user_id, message, None, None, None, None, None, false, false, None,
                )
                .await;

            results.push((user_id, result));
        }

        Ok(results)
    }

    /// Get conversation history
    pub async fn get_history(&self, peer_id: i64, limit: i32) -> VkResult<Vec<serde_json::Value>> {
        let response = self
            .api
            .messages_get_history(peer_id, 0, limit, None, true)
            .await?;

        let items = response["response"]["items"]
            .as_array()
            .ok_or_else(|| VkError::InvalidResponse("Expected array in response".to_string()))?
            .clone();

        Ok(items)
    }

    /// Get user information
    pub async fn get_user_info(
        &self,
        user_id: i64,
        fields: Option<&str>,
    ) -> VkResult<serde_json::Value> {
        let users = self.api.users_get(&[user_id], fields, None).await?;

        users["response"]
            .as_array()
            .and_then(|arr| arr.first())
            .cloned()
            .ok_or_else(|| VkError::InvalidResponse("User not found".to_string()))
    }

    /// Set typing indicator
    pub async fn set_typing(&self, peer_id: i64, user_id: Option<i64>) -> VkResult<()> {
        self.api
            .messages_set_activity(peer_id, user_id, "typing")
            .await?;
        Ok(())
    }

    /// Mark messages as read
    pub async fn mark_as_read(&self, peer_id: i64, start_message_id: Option<i64>) -> VkResult<()> {
        self.api
            .messages_mark_as_read(peer_id, start_message_id)
            .await?;
        Ok(())
    }
}
