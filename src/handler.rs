use crate::api::VkApi;
use crate::error::VkResult;
use crate::keyboard::Keyboard;
use crate::models::Event;
use async_trait::async_trait;

#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle(&self, event: &Event, api: &VkApi) -> VkResult<()>;
}

#[derive(Debug, Clone, Default)]
pub struct DefaultMessageHandler;

#[async_trait]
impl MessageHandler for DefaultMessageHandler {
    async fn handle(&self, event: &Event, api: &VkApi) -> VkResult<()> {
        match event {
            Event::MessageNew(message) => {
                let _ = api
                    .messages_mark_as_read(message.peer_id, Some(message.id))
                    .await;

                if message.text.starts_with('/') {
                    self.handle_command(message, api).await?;
                } else {
                    self.handle_text(message, api).await?;
                }

                if !message.attachments.is_empty() {
                    self.handle_attachments(message, api).await?;
                }

                if !message.fwd_messages.is_empty() {
                    self.handle_forwarded(message, api).await?;
                }
            }

            Event::MessageEdit(message) => {
                self.handle_edit(message, api).await?;
            }

            Event::MessageEvent(event) => {
                self.handle_message_event(event, api).await?;
            }

            Event::MessageTypingState(state) => {
                self.handle_typing(state, api).await?;
            }

            Event::MessageAllow(allow) => {
                self.handle_message_allow(allow, api).await?;
            }

            _ => {}
        }

        Ok(())
    }
}

impl DefaultMessageHandler {
    async fn handle_command(&self, message: &crate::models::Message, api: &VkApi) -> VkResult<()> {
        let command = message.text.split_whitespace().next().unwrap_or("");

        match command {
            "/start" | "/help" => {
                let keyboard = Keyboard::create_menu();
                api.messages_send(
                    message.peer_id,
                    "👋 Welcome! I'm a VK bot built with Rust.\n\nCommands:\n/start - Welcome message\n/help - Show help\n/menu - Show menu\n/info - Bot info",
                    Some(&keyboard),
                    None,
                    None,
                    None,
                    None,
                    false,
                    false,
                    None,
                ).await?;
            }

            "/menu" => {
                let keyboard = Keyboard::create_menu();
                api.messages_send(
                    message.peer_id,
                    "📱 Menu:",
                    Some(&keyboard),
                    None,
                    None,
                    None,
                    None,
                    false,
                    false,
                    None,
                )
                .await?;
            }

            "/info" => {
                api.messages_send(
                    message.peer_id,
                    "🤖 Bot built with vk-bot-api\n🦀 Powered by Rust\n⚡ Async/await\n🔧 Full VK API support",
                    None,
                    None,
                    None,
                    None,
                    None,
                    false,
                    false,
                    None,
                ).await?;
            }

            _ => {
                api.messages_send(
                    message.peer_id,
                    "❌ Unknown command. Use /help",
                    None,
                    None,
                    None,
                    None,
                    None,
                    false,
                    false,
                    None,
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn handle_text(&self, message: &crate::models::Message, api: &VkApi) -> VkResult<()> {
        let response = format!("You said: {}", message.text);

        api.messages_send(
            message.peer_id,
            &response,
            None,
            None,
            None,
            Some(message.id),
            None,
            false,
            false,
            None,
        )
        .await?;

        Ok(())
    }

    async fn handle_attachments(
        &self,
        message: &crate::models::Message,
        api: &VkApi,
    ) -> VkResult<()> {
        let mut types = Vec::new();

        for attachment in &message.attachments {
            match attachment.attachment_type.as_str() {
                "photo" => types.push("📷 Photo"),
                "video" => types.push("🎥 Video"),
                "audio" => types.push("🎵 Audio"),
                "doc" => types.push("📎 Document"),
                "sticker" => types.push("😊 Sticker"),
                _ => types.push("📎 Attachment"),
            }
        }

        if !types.is_empty() {
            let text = format!("Received attachments: {}", types.join(", "));

            api.messages_send(
                message.peer_id,
                &text,
                None,
                None,
                None,
                Some(message.id),
                None,
                false,
                false,
                None,
            )
            .await?;
        }

        Ok(())
    }

    async fn handle_forwarded(
        &self,
        message: &crate::models::Message,
        api: &VkApi,
    ) -> VkResult<()> {
        let count = message.fwd_messages.len();

        api.messages_send(
            message.peer_id,
            &format!("Forwarded messages: {}", count),
            None,
            None,
            None,
            Some(message.id),
            None,
            false,
            false,
            None,
        )
        .await?;

        Ok(())
    }

    async fn handle_edit(&self, message: &crate::models::Message, api: &VkApi) -> VkResult<()> {
        api.messages_send(
            message.peer_id,
            "Message was edited",
            None,
            None,
            None,
            None,
            None,
            true,
            false,
            None,
        )
        .await?;

        Ok(())
    }

    async fn handle_message_event(
        &self,
        event: &crate::models::MessageEvent,
        api: &VkApi,
    ) -> VkResult<()> {
        let _ = api
            .messages_send_message_event_answer(
                &event.event_id,
                event.user_id,
                event.peer_id,
                Some(r#"{"type": "show_snackbar", "text": "Done!"}"#),
            )
            .await;

        if let Some(payload) = &event.payload
            && let Some(command) = payload.get("command").and_then(|c| c.as_str())
            && command == "help"
        {
            api.messages_send(
                event.peer_id,
                "Help: Use /commands for available commands",
                None,
                None,
                None,
                None,
                None,
                false,
                false,
                None,
            )
            .await?;
        }

        Ok(())
    }

    async fn handle_typing(
        &self,
        _state: &crate::models::MessageTypingState,
        _api: &VkApi,
    ) -> VkResult<()> {
        Ok(())
    }

    async fn handle_message_allow(
        &self,
        allow: &crate::models::MessageAllow,
        api: &VkApi,
    ) -> VkResult<()> {
        api.messages_send(
            allow.user_id,
            "Thanks for allowing messages!",
            None,
            None,
            None,
            None,
            None,
            false,
            false,
            None,
        )
        .await?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AdminHandler {
    admin_ids: Vec<i64>,
}

impl AdminHandler {
    pub fn new(admin_ids: Vec<i64>) -> Self {
        Self { admin_ids }
    }

    pub fn is_admin(&self, user_id: i64) -> bool {
        self.admin_ids.contains(&user_id)
    }
}

#[async_trait]
impl MessageHandler for AdminHandler {
    async fn handle(&self, event: &Event, api: &VkApi) -> VkResult<()> {
        if let Event::MessageNew(message) = event
            && self.is_admin(message.from_id)
        {
            self.handle_admin_command(message, api).await?;
        }

        Ok(())
    }
}

impl AdminHandler {
    async fn handle_admin_command(
        &self,
        message: &crate::models::Message,
        api: &VkApi,
    ) -> VkResult<()> {
        if message.text.starts_with("/admin") {
            let keyboard = Keyboard::create_admin_menu();

            api.messages_send(
                message.peer_id,
                "🔧 Admin panel:",
                Some(&keyboard),
                None,
                None,
                None,
                None,
                false,
                false,
                None,
            )
            .await?;
        }

        Ok(())
    }
}
