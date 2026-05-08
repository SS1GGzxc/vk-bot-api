use crate::error::{VkError, VkResponseExt, VkResult};
use crate::keyboard::Keyboard;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

#[cfg(feature = "reqwest")]
use reqwest::Client;

#[cfg(feature = "uuid")]
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct VkApiConfig {
    pub version: String,
    pub timeout: u64,
    pub max_retries: u32,
    pub retry_delay: u64,
    pub enable_logging: bool,
    pub endpoint: String,
}

impl Default for VkApiConfig {
    fn default() -> Self {
        Self {
            version: "5.199".to_string(),
            timeout: 30,
            max_retries: 3,
            retry_delay: 1000,
            enable_logging: false,
            endpoint: "https://api.vk.com/method/".to_string(),
        }
    }
}

#[derive(Debug, Default)]
pub struct VkApiBuilder {
    token: Option<String>,
    config: VkApiConfig,
}

impl VkApiBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn token<T: Into<String>>(mut self, token: T) -> Self {
        self.token = Some(token.into());
        self
    }

    pub fn version<T: Into<String>>(mut self, version: T) -> Self {
        self.config.version = version.into();
        self
    }

    pub fn timeout(mut self, timeout: u64) -> Self {
        self.config.timeout = timeout;
        self
    }

    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.config.max_retries = max_retries;
        self
    }

    pub fn retry_delay(mut self, retry_delay: u64) -> Self {
        self.config.retry_delay = retry_delay;
        self
    }

    pub fn enable_logging(mut self, enable: bool) -> Self {
        self.config.enable_logging = enable;
        self
    }

    pub fn endpoint<T: Into<String>>(mut self, endpoint: T) -> Self {
        self.config.endpoint = endpoint.into();
        self
    }

    pub fn build(self) -> VkResult<VkApi> {
        let token = self
            .token
            .ok_or_else(|| VkError::ConfigError("Token is required".to_string()))?;

        #[cfg(feature = "reqwest")]
        let client = Client::builder()
            .timeout(Duration::from_secs(self.config.timeout))
            .pool_max_idle_per_host(10)
            .tcp_keepalive(Duration::from_secs(60))
            .build()
            .map_err(|e| VkError::NetworkError(e.to_string()))?;

        Ok(VkApi {
            #[cfg(feature = "reqwest")]
            client,
            token,
            config: self.config,
        })
    }
}

#[derive(Debug, Clone)]
pub struct VkApi {
    #[cfg(feature = "reqwest")]
    client: Client,
    token: String,
    config: VkApiConfig,
}

impl VkApi {
    pub fn new<T: Into<String>>(token: T) -> VkResult<Self> {
        VkApiBuilder::new().token(token).build()
    }

    pub fn builder() -> VkApiBuilder {
        VkApiBuilder::new()
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn version(&self) -> &str {
        &self.config.version
    }

    #[cfg(feature = "reqwest")]
    async fn call_method_with_retry(
        &self,
        method: &str,
        mut params: HashMap<String, String>,
    ) -> VkResult<Value> {
        let mut retries = 0;

        params.insert("access_token".to_string(), self.token.clone());
        params.insert("v".to_string(), self.config.version.clone());

        loop {
            match self.call_method_once(method, &params).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    retries += 1;

                    if retries >= self.config.max_retries || !self.should_retry(&e) {
                        return Err(e);
                    }

                    let delay_ms = self.config.retry_delay * 2u64.pow(retries - 1);
                    sleep(Duration::from_millis(delay_ms)).await;
                }
            }
        }
    }

    #[cfg(feature = "reqwest")]
    fn should_retry(&self, error: &VkError) -> bool {
        match error {
            VkError::RateLimit => true,
            VkError::NetworkError(_) => true,
            VkError::Timeout(_) => true,
            VkError::HttpError(e) => e
                .status()
                .map(|status| status.is_server_error() || status.as_u16() == 429)
                .unwrap_or(false),
            VkError::ApiError { code, .. } => {
                matches!(*code, 6 | 9 | 10 | 14)
            }
            _ => false,
        }
    }

    #[cfg(feature = "reqwest")]
    async fn call_method_once(
        &self,
        method: &str,
        params: &HashMap<String, String>,
    ) -> VkResult<Value> {
        let url = format!("{}{}", self.config.endpoint, method);

        if self.config.enable_logging {
            crate::vk_log!("API call: {} with {} params", method, params.len());
        }

        let response = self
            .client
            .post(&url)
            .form(params)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    VkError::Timeout(e.to_string())
                } else {
                    VkError::HttpError(e)
                }
            })?;

        self.handle_response(response).await
    }

    #[cfg(feature = "reqwest")]
    async fn handle_response(&self, response: reqwest::Response) -> VkResult<Value> {
        let status = response.status();

        if status == 429 {
            return Err(VkError::RateLimit);
        }

        if !status.is_success() {
            return Err(VkError::HttpError(
                response.error_for_status().err().unwrap(),
            ));
        }

        let json_response: Value = response.json().await?;

        if VkResponseExt::has_error(&json_response) {
            return VkResponseExt::extract_error(json_response);
        }

        Ok(json_response)
    }

    #[cfg(feature = "reqwest")]
    pub async fn call_method(
        &self,
        method: &str,
        params: HashMap<String, String>,
    ) -> VkResult<Value> {
        self.call_method_with_retry(method, params).await
    }

    #[cfg(not(feature = "reqwest"))]
    pub async fn call_method(
        &self,
        _method: &str,
        _params: HashMap<String, String>,
    ) -> VkResult<Value> {
        Err(VkError::ConfigError(
            "reqwest feature is required for API calls".to_string(),
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn messages_send(
        &self,
        peer_id: i64,
        message: &str,
        keyboard: Option<&Keyboard>,
        attachment: Option<&str>,
        sticker_id: Option<i64>,
        reply_to: Option<i64>,
        forward_messages: Option<&[i64]>,
        disable_mentions: bool,
        dont_parse_links: bool,
        random_id: Option<i64>,
    ) -> VkResult<i64> {
        let mut params = HashMap::new();
        params.insert("peer_id".to_string(), peer_id.to_string());
        params.insert("message".to_string(), message.to_string());

        #[cfg(feature = "uuid")]
        let random_id = random_id.unwrap_or_else(|| {
            let uuid = Uuid::new_v4();
            (uuid.as_u128() & 0x7FFFFFFFFFFFFFFF) as i64
        });

        #[cfg(not(feature = "uuid"))]
        let random_id = random_id.unwrap_or(0);

        params.insert("random_id".to_string(), random_id.to_string());

        if let Some(keyboard) = keyboard {
            params.insert("keyboard".to_string(), keyboard.to_json_string());
        }

        if let Some(attachment) = attachment {
            params.insert("attachment".to_string(), attachment.to_string());
        }

        if let Some(sticker_id) = sticker_id {
            params.insert("sticker_id".to_string(), sticker_id.to_string());
        }

        if let Some(reply_to) = reply_to {
            params.insert("reply_to".to_string(), reply_to.to_string());
        }

        if let Some(forward_messages) = forward_messages {
            let ids: Vec<String> = forward_messages.iter().map(|id| id.to_string()).collect();
            params.insert("forward_messages".to_string(), ids.join(","));
        }

        if disable_mentions {
            params.insert("disable_mentions".to_string(), "1".to_string());
        }

        if dont_parse_links {
            params.insert("dont_parse_links".to_string(), "1".to_string());
        }

        let response = self.call_method("messages.send", params).await?;
        let message_id = response["response"].as_i64().ok_or_else(|| {
            VkError::InvalidResponse("Expected message_id in response".to_string())
        })?;

        Ok(message_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn messages_edit(
        &self,
        peer_id: i64,
        message_id: i64,
        message: &str,
        keyboard: Option<&Keyboard>,
        attachment: Option<&str>,
        keep_forward_messages: bool,
        keep_snippets: bool,
    ) -> VkResult<bool> {
        let mut params = HashMap::new();
        params.insert("peer_id".to_string(), peer_id.to_string());
        params.insert("message_id".to_string(), message_id.to_string());
        params.insert("message".to_string(), message.to_string());

        if let Some(keyboard) = keyboard {
            params.insert("keyboard".to_string(), keyboard.to_json_string());
        }

        if let Some(attachment) = attachment {
            params.insert("attachment".to_string(), attachment.to_string());
        }

        if keep_forward_messages {
            params.insert("keep_forward_messages".to_string(), "1".to_string());
        }

        if keep_snippets {
            params.insert("keep_snippets".to_string(), "1".to_string());
        }

        let response = self.call_method("messages.edit", params).await?;
        Ok(response["response"].as_i64().unwrap_or(0) == 1)
    }

    pub async fn messages_delete(
        &self,
        message_ids: &[i64],
        delete_for_all: bool,
        spam: bool,
    ) -> VkResult<serde_json::Map<String, Value>> {
        let mut params = HashMap::new();
        let ids: Vec<String> = message_ids.iter().map(|id| id.to_string()).collect();
        params.insert("message_ids".to_string(), ids.join(","));

        if delete_for_all {
            params.insert("delete_for_all".to_string(), "1".to_string());
        }

        if spam {
            params.insert("spam".to_string(), "1".to_string());
        }

        let response = self.call_method("messages.delete", params).await?;
        Ok(response["response"]
            .as_object()
            .ok_or_else(|| VkError::InvalidResponse("Expected object in response".to_string()))?
            .clone())
    }

    pub async fn messages_restore(&self, message_id: i64) -> VkResult<bool> {
        let mut params = HashMap::new();
        params.insert("message_id".to_string(), message_id.to_string());

        let response = self.call_method("messages.restore", params).await?;
        Ok(response["response"].as_i64().unwrap_or(0) == 1)
    }

    pub async fn messages_mark_as_read(
        &self,
        peer_id: i64,
        start_message_id: Option<i64>,
    ) -> VkResult<bool> {
        let mut params = HashMap::new();
        params.insert("peer_id".to_string(), peer_id.to_string());

        if let Some(start_id) = start_message_id {
            params.insert("start_message_id".to_string(), start_id.to_string());
        }

        let response = self.call_method("messages.markAsRead", params).await?;
        Ok(response["response"].as_i64().unwrap_or(0) == 1)
    }

    pub async fn messages_mark_as_important(
        &self,
        message_ids: &[i64],
        important: Option<i32>,
    ) -> VkResult<Vec<i64>> {
        let mut params = HashMap::new();
        let ids: Vec<String> = message_ids.iter().map(|id| id.to_string()).collect();
        params.insert("message_ids".to_string(), ids.join(","));

        if let Some(important) = important {
            params.insert("important".to_string(), important.to_string());
        }

        let response = self.call_method("messages.markAsImportant", params).await?;
        let marked_ids: Vec<i64> = response["response"]
            .as_array()
            .ok_or_else(|| VkError::InvalidResponse("Expected array in response".to_string()))?
            .iter()
            .filter_map(|v| v.as_i64())
            .collect();

        Ok(marked_ids)
    }

    pub async fn messages_get_conversations(
        &self,
        offset: i32,
        count: i32,
        filter: Option<&str>,
    ) -> VkResult<Value> {
        let mut params = HashMap::new();
        params.insert("offset".to_string(), offset.to_string());
        params.insert("count".to_string(), count.to_string());

        if let Some(filter) = filter {
            params.insert("filter".to_string(), filter.to_string());
        }

        self.call_method("messages.getConversations", params).await
    }

    pub async fn messages_get_conversation_members(
        &self,
        peer_id: i64,
        fields: Option<&str>,
    ) -> VkResult<Value> {
        let mut params = HashMap::new();
        params.insert("peer_id".to_string(), peer_id.to_string());

        if let Some(fields) = fields {
            params.insert("fields".to_string(), fields.to_string());
        }

        self.call_method("messages.getConversationMembers", params)
            .await
    }

    pub async fn messages_get_history(
        &self,
        peer_id: i64,
        offset: i32,
        count: i32,
        start_message_id: Option<i64>,
        rev: bool,
    ) -> VkResult<Value> {
        let mut params = HashMap::new();
        params.insert("peer_id".to_string(), peer_id.to_string());
        params.insert("offset".to_string(), offset.to_string());
        params.insert("count".to_string(), count.to_string());

        if let Some(start_id) = start_message_id {
            params.insert("start_message_id".to_string(), start_id.to_string());
        }

        if rev {
            params.insert("rev".to_string(), "1".to_string());
        }

        self.call_method("messages.getHistory", params).await
    }

    pub async fn messages_get_by_id(
        &self,
        message_ids: &[i64],
        preview_length: i32,
        extended: bool,
    ) -> VkResult<Value> {
        let mut params = HashMap::new();
        let ids: Vec<String> = message_ids.iter().map(|id| id.to_string()).collect();
        params.insert("message_ids".to_string(), ids.join(","));
        params.insert("preview_length".to_string(), preview_length.to_string());

        if extended {
            params.insert("extended".to_string(), "1".to_string());
        }

        self.call_method("messages.getById", params).await
    }

    pub async fn messages_search(
        &self,
        query: &str,
        peer_id: Option<i64>,
        date: Option<i64>,
        count: i32,
    ) -> VkResult<Value> {
        let mut params = HashMap::new();
        params.insert("q".to_string(), query.to_string());
        params.insert("count".to_string(), count.to_string());

        if let Some(peer_id) = peer_id {
            params.insert("peer_id".to_string(), peer_id.to_string());
        }

        if let Some(date) = date {
            params.insert("date".to_string(), date.to_string());
        }

        self.call_method("messages.search", params).await
    }

    pub async fn messages_search_conversations(
        &self,
        query: &str,
        count: i32,
        extended: bool,
        fields: Option<&str>,
    ) -> VkResult<Value> {
        let mut params = HashMap::new();
        params.insert("q".to_string(), query.to_string());
        params.insert("count".to_string(), count.to_string());

        if extended {
            params.insert("extended".to_string(), "1".to_string());
        }

        if let Some(fields) = fields {
            params.insert("fields".to_string(), fields.to_string());
        }

        self.call_method("messages.searchConversations", params)
            .await
    }

    pub async fn messages_get_attachments(
        &self,
        peer_id: i64,
        media_type: &str,
        start_from: Option<&str>,
        count: i32,
    ) -> VkResult<Value> {
        let mut params = HashMap::new();
        params.insert("peer_id".to_string(), peer_id.to_string());
        params.insert("media_type".to_string(), media_type.to_string());
        params.insert("count".to_string(), count.to_string());

        if let Some(start_from) = start_from {
            params.insert("start_from".to_string(), start_from.to_string());
        }

        self.call_method("messages.getAttachments", params).await
    }

    pub async fn messages_get_invite_link(&self, peer_id: i64, reset: bool) -> VkResult<String> {
        let mut params = HashMap::new();
        params.insert("peer_id".to_string(), peer_id.to_string());

        if reset {
            params.insert("reset".to_string(), "1".to_string());
        }

        let response = self.call_method("messages.getInviteLink", params).await?;
        let link = response["response"]["link"]
            .as_str()
            .ok_or_else(|| VkError::MissingField("link".to_string()))?
            .to_string();

        Ok(link)
    }

    pub async fn messages_remove_chat_user(
        &self,
        chat_id: i64,
        user_id: i64,
        member_id: Option<i64>,
    ) -> VkResult<bool> {
        let mut params = HashMap::new();
        params.insert("chat_id".to_string(), chat_id.to_string());
        params.insert("user_id".to_string(), user_id.to_string());

        if let Some(member_id) = member_id {
            params.insert("member_id".to_string(), member_id.to_string());
        }

        let response = self.call_method("messages.removeChatUser", params).await?;
        Ok(response["response"].as_i64().unwrap_or(0) == 1)
    }

    pub async fn messages_add_chat_user(&self, chat_id: i64, user_id: i64) -> VkResult<bool> {
        let mut params = HashMap::new();
        params.insert("chat_id".to_string(), chat_id.to_string());
        params.insert("user_id".to_string(), user_id.to_string());

        let response = self.call_method("messages.addChatUser", params).await?;
        Ok(response["response"].as_i64().unwrap_or(0) == 1)
    }

    pub async fn messages_create_chat(
        &self,
        user_ids: &[i64],
        title: Option<&str>,
    ) -> VkResult<i64> {
        let mut params = HashMap::new();
        let ids: Vec<String> = user_ids.iter().map(|id| id.to_string()).collect();
        params.insert("user_ids".to_string(), ids.join(","));

        if let Some(title) = title {
            params.insert("title".to_string(), title.to_string());
        }

        let response = self.call_method("messages.createChat", params).await?;
        let chat_id = response["response"]
            .as_i64()
            .ok_or_else(|| VkError::InvalidResponse("Expected chat_id in response".to_string()))?;

        Ok(chat_id)
    }

    pub async fn messages_set_activity(
        &self,
        peer_id: i64,
        user_id: Option<i64>,
        activity_type: &str,
    ) -> VkResult<bool> {
        let mut params = HashMap::new();
        params.insert("peer_id".to_string(), peer_id.to_string());
        params.insert("type".to_string(), activity_type.to_string());

        if let Some(user_id) = user_id {
            params.insert("user_id".to_string(), user_id.to_string());
        }

        let response = self.call_method("messages.setActivity", params).await?;
        Ok(response["response"].as_i64().unwrap_or(0) == 1)
    }

    pub async fn messages_send_message_event_answer(
        &self,
        event_id: &str,
        user_id: i64,
        peer_id: i64,
        event_data: Option<&str>,
    ) -> VkResult<bool> {
        let mut params = HashMap::new();
        params.insert("event_id".to_string(), event_id.to_string());
        params.insert("user_id".to_string(), user_id.to_string());
        params.insert("peer_id".to_string(), peer_id.to_string());

        if let Some(event_data) = event_data {
            params.insert("event_data".to_string(), event_data.to_string());
        }

        let response = self
            .call_method("messages.sendMessageEventAnswer", params)
            .await?;
        Ok(response["response"].as_i64().unwrap_or(0) == 1)
    }

    pub async fn users_get(
        &self,
        user_ids: &[i64],
        fields: Option<&str>,
        name_case: Option<&str>,
    ) -> VkResult<Value> {
        let mut params = HashMap::new();
        let ids: Vec<String> = user_ids.iter().map(|id| id.to_string()).collect();
        params.insert("user_ids".to_string(), ids.join(","));

        if let Some(fields) = fields {
            params.insert("fields".to_string(), fields.to_string());
        }

        if let Some(name_case) = name_case {
            params.insert("name_case".to_string(), name_case.to_string());
        }

        self.call_method("users.get", params).await
    }

    pub async fn groups_get_long_poll_server(&self, group_id: i64) -> VkResult<LongPollServer> {
        let mut params = HashMap::new();
        params.insert("group_id".to_string(), group_id.to_string());

        let response = self.call_method("groups.getLongPollServer", params).await?;

        let server = response["response"]["server"]
            .as_str()
            .ok_or_else(|| VkError::MissingField("server".to_string()))?
            .to_string();

        let key = response["response"]["key"]
            .as_str()
            .ok_or_else(|| VkError::MissingField("key".to_string()))?
            .to_string();

        let ts = response["response"]["ts"]
            .as_str()
            .ok_or_else(|| VkError::MissingField("ts".to_string()))?
            .to_string();

        Ok(LongPollServer { server, key, ts })
    }

    pub async fn groups_get_by_id(
        &self,
        group_ids: &[i64],
        fields: Option<&str>,
    ) -> VkResult<Value> {
        let mut params = HashMap::new();
        let ids: Vec<String> = group_ids.iter().map(|id| id.to_string()).collect();
        params.insert("group_ids".to_string(), ids.join(","));

        if let Some(fields) = fields {
            params.insert("fields".to_string(), fields.to_string());
        }

        self.call_method("groups.getById", params).await
    }

    pub async fn send_message(&self, peer_id: i64, message: &str) -> VkResult<i64> {
        self.messages_send(
            peer_id, message, None, None, None, None, None, false, false, None,
        )
        .await
    }

    pub async fn send_message_with_keyboard(
        &self,
        peer_id: i64,
        message: &str,
        keyboard: &Keyboard,
    ) -> VkResult<i64> {
        self.messages_send(
            peer_id,
            message,
            Some(keyboard),
            None,
            None,
            None,
            None,
            false,
            false,
            None,
        )
        .await
    }

    pub async fn send_reply(&self, peer_id: i64, message: &str, reply_to: i64) -> VkResult<i64> {
        self.messages_send(
            peer_id,
            message,
            None,
            None,
            None,
            Some(reply_to),
            None,
            false,
            false,
            None,
        )
        .await
    }

    // ==================== MEDIA UPLOAD METHODS ====================

    /// Get upload server for photos
    pub async fn photos_get_messages_upload_server(&self, peer_id: i64) -> VkResult<UploadServer> {
        let mut params = HashMap::new();
        params.insert("peer_id".to_string(), peer_id.to_string());

        let response = self
            .call_method("photos.getMessagesUploadServer", params)
            .await?;

        let upload_url = response["response"]["upload_url"]
            .as_str()
            .ok_or_else(|| VkError::MissingField("upload_url".to_string()))?
            .to_string();

        Ok(UploadServer { upload_url })
    }

    /// Save uploaded photo
    pub async fn photos_save_messages_photo(
        &self,
        photo: &str,
        server: i64,
        hash: &str,
    ) -> VkResult<Vec<SavedPhoto>> {
        let mut params = HashMap::new();
        params.insert("photo".to_string(), photo.to_string());
        params.insert("server".to_string(), server.to_string());
        params.insert("hash".to_string(), hash.to_string());

        let response = self.call_method("photos.saveMessagesPhoto", params).await?;

        let photos: Vec<SavedPhoto> =
            serde_json::from_value(response["response"].clone()).map_err(VkError::JsonError)?;

        Ok(photos)
    }

    /// Get upload server for documents
    pub async fn docs_get_upload_server(
        &self,
        peer_id: Option<i64>,
        doc_type: Option<&str>,
    ) -> VkResult<UploadServer> {
        let mut params = HashMap::new();

        if let Some(peer_id) = peer_id {
            params.insert("peer_id".to_string(), peer_id.to_string());
        }

        if let Some(doc_type) = doc_type {
            params.insert("type".to_string(), doc_type.to_string());
        }

        let response = self.call_method("docs.getUploadServer", params).await?;

        let upload_url = response["response"]["upload_url"]
            .as_str()
            .ok_or_else(|| VkError::MissingField("upload_url".to_string()))?
            .to_string();

        Ok(UploadServer { upload_url })
    }

    /// Get upload server for documents (for community messages)
    pub async fn docs_get_messages_upload_server(
        &self,
        peer_id: i64,
        doc_type: Option<&str>,
    ) -> VkResult<UploadServer> {
        let mut params = HashMap::new();
        params.insert("peer_id".to_string(), peer_id.to_string());

        if let Some(doc_type) = doc_type {
            params.insert("type".to_string(), doc_type.to_string());
        }

        let response = self
            .call_method("docs.getMessagesUploadServer", params)
            .await?;

        let upload_url = response["response"]["upload_url"]
            .as_str()
            .ok_or_else(|| VkError::MissingField("upload_url".to_string()))?
            .to_string();

        Ok(UploadServer { upload_url })
    }

    /// Save uploaded document
    pub async fn docs_save(
        &self,
        file: &str,
        title: Option<&str>,
        tags: Option<&str>,
    ) -> VkResult<SavedDocument> {
        let mut params = HashMap::new();
        params.insert("file".to_string(), file.to_string());

        if let Some(title) = title {
            params.insert("title".to_string(), title.to_string());
        }

        if let Some(tags) = tags {
            params.insert("tags".to_string(), tags.to_string());
        }

        let response = self.call_method("docs.save", params).await?;

        crate::vk_log!("docs.save response: {}", response);

        let doc_response: DocSaveResponse =
            serde_json::from_value(response["response"].clone()).map_err(VkError::JsonError)?;

        crate::vk_log!(
            "Parsed doc response: type={:?}, has_doc={}, has_audio_msg={}",
            doc_response.doc_type,
            doc_response.doc.is_some(),
            doc_response.audio_message.is_some()
        );

        Ok(SavedDocument {
            doc: doc_response.doc,
            audio_message: doc_response.audio_message,
        })
    }

    // ==================== UPLOAD HELPERS ====================

    /// Upload file to VK server
    #[cfg(feature = "reqwest")]
    pub async fn upload_file(
        &self,
        upload_url: &str,
        file_data: Vec<u8>,
        filename: &str,
    ) -> VkResult<UploadResponse> {
        // Detect content type from filename extension
        let content_type = Self::detect_content_type(filename);

        let mut part = reqwest::multipart::Part::bytes(file_data).file_name(filename.to_string());

        if let Some(ct) = content_type {
            part = part
                .mime_str(&ct)
                .map_err(|e| VkError::InternalError(format!("Invalid MIME type: {}", e)))?;
        }

        let form = reqwest::multipart::Form::new().part("file", part);

        crate::vk_log!("Uploading file '{}' to {}", filename, upload_url);

        let response = self
            .client
            .post(upload_url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| VkError::NetworkError(format!("Upload request failed: {}", e)))?;

        let status = response.status();

        // Get raw response text
        let response_text = response
            .text()
            .await
            .map_err(|e| VkError::NetworkError(format!("Failed to read response: {}", e)))?;

        crate::vk_log!("Upload response (status: {}): {}", status, response_text);

        if !status.is_success() {
            return Err(VkError::NetworkError(format!(
                "Upload failed with status {}: {}",
                status, response_text
            )));
        }

        // Parse the JSON response
        let upload_response: UploadResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                VkError::InvalidResponse(format!(
                    "Failed to parse upload response: {}. Raw response: {}",
                    e, response_text
                ))
            })?;

        Ok(upload_response)
    }

    /// Detect MIME type from filename extension
    fn detect_content_type(filename: &str) -> Option<String> {
        let ext = filename.split('.').next_back()?.to_lowercase();
        match ext.as_str() {
            "jpg" | "jpeg" => Some("image/jpeg".to_string()),
            "png" => Some("image/png".to_string()),
            "gif" => Some("image/gif".to_string()),
            "pdf" => Some("application/pdf".to_string()),
            "txt" => Some("text/plain".to_string()),
            "ogg" => Some("audio/ogg".to_string()),
            "mp3" => Some("audio/mpeg".to_string()),
            "mp4" => Some("video/mp4".to_string()),
            _ => None,
        }
    }

    /// Upload and send photo
    pub async fn send_photo(
        &self,
        peer_id: i64,
        photo_data: Vec<u8>,
        filename: &str,
        caption: Option<&str>,
    ) -> VkResult<i64> {
        // Step 1: Get upload server
        let upload_server = self.photos_get_messages_upload_server(peer_id).await?;

        // Step 2: Upload photo
        let upload_response = self
            .upload_file(&upload_server.upload_url, photo_data, filename)
            .await?;

        // Validate upload response for photos
        if upload_response.server == 0
            || upload_response.photo.is_empty()
            || upload_response.hash.is_empty()
        {
            return Err(VkError::InvalidResponse(
                "Upload response missing photo data. Server, photo, or hash is empty".to_string(),
            ));
        }

        crate::vk_log!(
            "Photo upload successful: server={}, photo length={}",
            upload_response.server,
            upload_response.photo.len()
        );

        // Step 3: Save photo
        let saved_photos = self
            .photos_save_messages_photo(
                &upload_response.photo,
                upload_response.server,
                &upload_response.hash,
            )
            .await?;

        // Step 4: Create attachment string
        let attachment = saved_photos
            .first()
            .map(|p| format!("photo{}_{}", p.owner_id, p.id))
            .ok_or_else(|| VkError::InvalidResponse("No photo saved".to_string()))?;

        // Step 5: Send message with photo
        let message = caption.unwrap_or("");
        self.messages_send(
            peer_id,
            message,
            None,
            Some(&attachment),
            None,
            None,
            None,
            false,
            false,
            None,
        )
        .await
    }

    /// Upload and send document
    pub async fn send_document(
        &self,
        peer_id: i64,
        file_data: Vec<u8>,
        filename: &str,
        title: Option<&str>,
        caption: Option<&str>,
    ) -> VkResult<i64> {
        // Step 1: Get upload server
        let upload_server = self.docs_get_messages_upload_server(peer_id, None).await?;

        // Step 2: Upload file
        let upload_response = self
            .upload_file(&upload_server.upload_url, file_data, filename)
            .await?;

        // Validate upload response
        if upload_response.file.is_empty() {
            return Err(VkError::InvalidResponse(
                "Upload response missing file data".to_string(),
            ));
        }

        crate::vk_log!(
            "Document upload successful: file field length={}",
            upload_response.file.len()
        );

        // Step 3: Save document
        let saved_doc = self
            .docs_save(&upload_response.file, title.or(Some(filename)), None)
            .await?;

        // Step 4: Create attachment string
        let doc = saved_doc
            .doc
            .ok_or_else(|| VkError::InvalidResponse("No document saved".to_string()))?;
        let attachment = format!("doc{}_{}", doc.owner_id, doc.id);

        // Step 5: Send message with document
        let message = caption.unwrap_or("");
        self.messages_send(
            peer_id,
            message,
            None,
            Some(&attachment),
            None,
            None,
            None,
            false,
            false,
            None,
        )
        .await
    }

    /// Upload and send audio message (voice message)
    pub async fn send_voice_message(
        &self,
        peer_id: i64,
        audio_data: Vec<u8>,
        filename: &str,
    ) -> VkResult<i64> {
        // Step 1: Get upload server for audio messages
        let upload_server = self
            .docs_get_messages_upload_server(peer_id, Some("audio_message"))
            .await?;

        // Step 2: Upload file
        let upload_response = self
            .upload_file(&upload_server.upload_url, audio_data, filename)
            .await?;

        // Validate upload response
        if upload_response.file.is_empty() {
            return Err(VkError::InvalidResponse(
                "Upload response missing file data".to_string(),
            ));
        }

        crate::vk_log!(
            "Voice message upload successful: file field length={}",
            upload_response.file.len()
        );

        // Step 3: Save document
        let saved_doc = self
            .docs_save(&upload_response.file, Some(filename), None)
            .await?;

        // Step 4: Check if audio_message is present
        if saved_doc.audio_message.is_none() {
            return Err(VkError::InvalidResponse(
                "Audio message was not processed".to_string(),
            ));
        }

        // Step 5: Create attachment string from doc
        let doc = saved_doc
            .doc
            .ok_or_else(|| VkError::InvalidResponse("No document saved".to_string()))?;
        let attachment = format!("doc{}_{}", doc.owner_id, doc.id);

        // Step 6: Send message with audio
        self.messages_send(
            peer_id,
            "",
            None,
            Some(&attachment),
            None,
            None,
            None,
            false,
            false,
            None,
        )
        .await
    }

    // ==================== DOWNLOAD METHODS ====================

    /// Download file from URL
    #[cfg(feature = "reqwest")]
    pub async fn download_file(&self, url: &str) -> VkResult<DownloadedFile> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| VkError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(VkError::NetworkError(format!(
                "Download failed: {}",
                response.status()
            )));
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let data = response
            .bytes()
            .await
            .map_err(|e| VkError::NetworkError(e.to_string()))?;

        Ok(DownloadedFile {
            data: data.to_vec(),
            content_type,
        })
    }

    /// Download photo by attachment
    pub async fn download_photo(&self, photo: &crate::models::Photo) -> VkResult<DownloadedFile> {
        // Get the largest available size
        let size = photo
            .sizes
            .iter()
            .max_by_key(|s| s.width * s.height)
            .ok_or_else(|| VkError::InvalidResponse("Photo has no sizes".to_string()))?;

        self.download_file(&size.url).await
    }

    /// Download document
    pub async fn download_document(
        &self,
        doc: &crate::models::Document,
    ) -> VkResult<DownloadedFile> {
        self.download_file(&doc.url).await
    }

    /// Download audio
    pub async fn download_audio(&self, audio: &crate::models::Audio) -> VkResult<DownloadedFile> {
        let url = audio
            .url
            .as_ref()
            .ok_or_else(|| VkError::InvalidResponse("Audio has no URL".to_string()))?;

        self.download_file(url).await
    }

    /// Download video thumbnail
    pub async fn download_video_thumbnail(
        &self,
        video: &crate::models::Video,
    ) -> VkResult<DownloadedFile> {
        let image = video
            .image
            .first()
            .ok_or_else(|| VkError::InvalidResponse("Video has no thumbnails".to_string()))?;

        self.download_file(&image.url).await
    }

    /// Download sticker
    pub async fn download_sticker(
        &self,
        sticker: &crate::models::Sticker,
    ) -> VkResult<DownloadedFile> {
        let image = sticker
            .images
            .first()
            .ok_or_else(|| VkError::InvalidResponse("Sticker has no images".to_string()))?;

        self.download_file(&image.url).await
    }

    /// Download audio message
    pub async fn download_audio_message(
        &self,
        audio_msg: &crate::models::AudioMessage,
    ) -> VkResult<DownloadedFile> {
        // Prefer MP3 over OGG for better compatibility
        self.download_file(&audio_msg.link_mp3).await
    }

    /// Forward downloaded file back to another peer
    pub async fn forward_attachment<A: AsRef<str>>(
        &self,
        peer_id: i64,
        attachment: A,
        caption: Option<&str>,
    ) -> VkResult<i64> {
        self.messages_send(
            peer_id,
            caption.unwrap_or(""),
            None,
            Some(attachment.as_ref()),
            None,
            None,
            None,
            false,
            false,
            None,
        )
        .await
    }
}

/// Long Poll server information
#[derive(Debug, Clone)]
pub struct LongPollServer {
    pub server: String,
    pub key: String,
    pub ts: String,
}

/// Upload server information
#[derive(Debug, Clone)]
pub struct UploadServer {
    pub upload_url: String,
}

/// Upload response from VK
///
/// Note: VK returns different fields depending on upload type:
/// - Photo uploads: server, photo, hash
/// - Document uploads: file
#[derive(Debug, Clone, Deserialize)]
pub struct UploadResponse {
    #[serde(default)]
    pub server: i64,
    #[serde(default)]
    pub photo: String,
    #[serde(default)]
    pub hash: String,
    #[serde(default)]
    pub file: String,
}

/// Saved photo information
#[derive(Debug, Clone, Deserialize)]
pub struct SavedPhoto {
    pub id: i64,
    pub owner_id: i64,
    #[serde(default)]
    pub access_key: Option<String>,
    #[serde(default)]
    pub sizes: Vec<PhotoSizeInfo>,
}

/// Photo size information
#[derive(Debug, Clone, Deserialize)]
pub struct PhotoSizeInfo {
    #[serde(rename = "type")]
    pub size_type: String,
    pub url: String,
    pub width: i32,
    pub height: i32,
}

/// Document information from save response
#[derive(Debug, Clone, Deserialize)]
pub struct DocInfo {
    pub id: i64,
    pub owner_id: i64,
    pub title: String,
    pub size: i64,
    pub ext: String,
    pub url: String,
    pub date: i64,
    #[serde(rename = "type")]
    pub doc_type: i32,
}

/// Audio message information from save response
#[derive(Debug, Clone, Deserialize)]
pub struct AudioMessageInfo {
    pub duration: i32,
    #[serde(default)]
    pub waveform: Vec<i32>,
    pub link_ogg: String,
    pub link_mp3: String,
}

/// Saved document response
#[derive(Debug, Clone)]
pub struct SavedDocument {
    pub doc: Option<DocInfo>,
    pub audio_message: Option<AudioMessageInfo>,
}

/// Response structure from docs.save
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct DocSaveResponse {
    #[serde(rename = "type")]
    pub doc_type: Option<String>,
    pub doc: Option<DocInfo>,
    #[serde(rename = "audio_message")]
    pub audio_message: Option<AudioMessageInfo>,
}

/// Downloaded file information
#[derive(Debug, Clone)]
pub struct DownloadedFile {
    pub data: Vec<u8>,
    pub content_type: Option<String>,
}
