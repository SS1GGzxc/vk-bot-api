use async_trait::async_trait;
use std::error::Error;
use std::sync::Arc;
use vk_bot_api::prelude::*;

/// MediaBotHandler - demonstrates upload and download functionality
#[derive(Debug, Clone)]
struct MediaBotHandler {
    // Track statistics
    stats: Arc<std::sync::atomic::AtomicU64>,
}

impl Default for MediaBotHandler {
    fn default() -> Self {
        Self {
            stats: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }
}

#[async_trait]
impl MessageHandler for MediaBotHandler {
    async fn handle(&self, event: &Event, api: &VkApi) -> VkResult<()> {
        match event {
            Event::MessageNew(message) => {
                self.stats.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                let text = message.text.to_lowercase();

                // Handle commands
                if text.starts_with("/help") || text.starts_with("/start") {
                    self.send_help(message.peer_id, api).await?;
                } else if text.starts_with("/photo") {
                    self.send_example_photo(message.peer_id, api).await?;
                } else if text.starts_with("/doc") {
                    self.send_example_document(message.peer_id, api).await?;
                } else if text.starts_with("/voice") {
                    self.send_example_voice(message.peer_id, api).await?;
                } else if text.starts_with("/stats") {
                    self.send_stats(message.peer_id, api).await?;
                } else {
                    // Check for incoming attachments and download them
                    self.handle_incoming_attachments(message, api).await?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl MediaBotHandler {
    /// Send help message with available commands
    async fn send_help(&self, peer_id: i64, api: &VkApi) -> VkResult<()> {
        let help_text = "📱 Media Bot Commands:\n\n\
            /help - Show this help\n\
            /photo - Send example photo\n\
            /doc - Send example document\n\
            /voice - Send example voice message\n\
            /stats - Show bot statistics\n\n\
            📎 You can also send me:\n\
            • Photos - I'll download them\n\
            • Documents - I'll save them\n\
            • Voice messages - I'll process them\n\
            • Any attachment - I'll try to download it";

        api.send_message(peer_id, help_text).await?;
        Ok(())
    }

    /// Send an example photo (try to load from file or send instructions)
    async fn send_example_photo(&self, peer_id: i64, api: &VkApi) -> VkResult<()> {
        // Try to load a real image file from disk
        let photo_path = "example_photo.jpg";
        
        match std::fs::read(photo_path) {
            Ok(photo_data) => {
                // Send the actual image file
                api.send_photo(
                    peer_id,
                    photo_data,
                    "example.jpg",
                    Some("📷 Here is an example photo!")
                ).await?;
            }
            Err(_) => {
                // File not found - send instructions
                api.send_message(
                    peer_id,
                    "📷 To test photo upload, please create an image file named 'example_photo.jpg' in the bot directory.\n\n\
                        Or send me a photo and I'll download and forward it back to you!"
                ).await?;
            }
        }

        Ok(())
    }

    /// Send an example document
    async fn send_example_document(&self, peer_id: i64, api: &VkApi) -> VkResult<()> {
        // Create a simple text document
        let doc_content = b"Hello from VK Bot API!\n\n\
            This is an example document sent using the media upload features.\n\n\
            Features demonstrated:\n\
            - Document upload\n\
            - Custom title\n\
            - Message caption\n\n\
            Sent with <3 by vk-bot-api";

        api.send_document(
            peer_id,
            doc_content.to_vec(),
            "example.txt",
            Some("Example Document"),
            Some("📄 Here is an example document!")
        ).await?;

        Ok(())
    }

    /// Send an example voice message (with placeholder audio)
    async fn send_example_voice(&self, peer_id: i64, api: &VkApi) -> VkResult<()> {
        // Note: In a real application, you would load an actual OGG audio file
        // For this example, we'll send a text message explaining
        api.send_message(
            peer_id,
            "\u{1F3A4} To send voice messages, use:\n\n\
                api.send_voice_message(peer_id, audio_data, \"voice.ogg\").await?;\n\n\
                The audio must be in OGG Opus format."
        ).await?;

        Ok(())
    }

    /// Send bot statistics
    async fn send_stats(&self, peer_id: i64, api: &VkApi) -> VkResult<()> {
        let count = self.stats.load(std::sync::atomic::Ordering::SeqCst);

        let stats_text = format!(
            "\u{1F4CA} Bot Statistics:\n\n\
                Messages processed: {}\n\
                Media features: Enabled\n\
                Upload support: Photos, Documents, Voice\n\
                Download support: All attachments",
            count
        );

        api.send_message(peer_id, &stats_text).await?;
        Ok(())
    }

    /// Handle incoming attachments by downloading them
    async fn handle_incoming_attachments(
        &self,
        message: &vk_bot_api::models::Message,
        api: &VkApi,
    ) -> VkResult<()> {
        if message.attachments.is_empty() {
            // No attachments - respond to regular text
            if !message.text.is_empty() && !message.text.starts_with('/') {
                api.send_message(
                    message.peer_id,
                    &format!("You said: {}\n\nTry /help for available commands!", message.text)
                ).await?;
            }
            return Ok(());
        }

        // Process each attachment
        for (idx, attachment) in message.attachments.iter().enumerate() {
            match attachment.attachment_type.as_str() {
                "photo" => {
                    log::info!("Processing photo attachment #{}: owner_id={}, id={}, has_access_key={}", 
                        idx + 1,
                        attachment.photo.as_ref().map(|p| p.owner_id).unwrap_or(0),
                        attachment.photo.as_ref().map(|p| p.id).unwrap_or(0),
                        attachment.photo.as_ref().map(|p| p.access_key.is_some()).unwrap_or(false)
                    );
                    
                    if let Some(photo) = &attachment.photo {
                        api.send_message(
                            message.peer_id,
                            &format!("\u{1F4F7} Downloading photo #{}...", idx + 1)
                        ).await?;

                        match api.download_photo(photo).await {
                            Ok(downloaded) => {
                                let size_kb = downloaded.data.len() / 1024;
                                
                                // Save photo locally
                                let filename = format!("received_photo_{}_{}.jpg", photo.owner_id, photo.id);
                                match std::fs::write(&filename, &downloaded.data) {
                                    Ok(_) => {
                                        log::info!("Photo saved to: {}", filename);
                                    }
                                    Err(e) => {
                                        log::error!("Failed to save photo: {}", e);
                                    }
                                }
                                
                                api.send_message(
                                    message.peer_id,
                                    &format!(
                                        "\u{2705} Photo downloaded!\n\
                                            Size: {} KB\n\
                                            Type: {:?}\n\
                                            Saved as: {}",
                                        size_kb,
                                        downloaded.content_type.unwrap_or_default(),
                                        filename
                                    )
                                ).await?;

                                // Upload and send the photo back
                                log::info!("Re-uploading photo with {} bytes", downloaded.data.len());
                                
                                match api.send_photo(
                                    message.peer_id,
                                    downloaded.data,
                                    "photo.jpg",
                                    Some("\u{1F504} Here is your photo re-uploaded!")
                                ).await {
                                    Ok(message_id) => {
                                        log::info!("Photo re-uploaded successfully, message_id: {}", message_id);
                                        api.send_message(
                                            message.peer_id,
                                            &format!("\u{2705} Photo sent back! Message ID: {}", message_id)
                                        ).await?;
                                    }
                                    Err(e) => {
                                        log::error!("Failed to re-upload photo: {}", e);
                                        api.send_message(
                                            message.peer_id,
                                            &format!("\u{26A0} Downloaded but failed to send back: {}", e)
                                        ).await?;
                                    }
                                }
                            }
                            Err(e) => {
                                api.send_message(
                                    message.peer_id,
                                    &format!("\u{274C} Failed to download photo: {}", e)
                                ).await?;
                            }
                        }
                    }
                }

                "doc" => {
                    if let Some(doc) = &attachment.doc {
                        api.send_message(
                            message.peer_id,
                            &format!("\u{1F4C4} Downloading document: {}...", doc.title)
                        ).await?;

                        match api.download_document(doc).await {
                            Ok(downloaded) => {
                                let size_kb = downloaded.data.len() / 1024;
                                api.send_message(
                                    message.peer_id,
                                    &format!(
                                        "\u{2705} Document downloaded!\n\
                                            Title: {}\n\
                                            Size: {} KB\n\
                                            Extension: {}",
                                        doc.title, size_kb, doc.ext
                                    )
                                ).await?;
                            }
                            Err(e) => {
                                api.send_message(
                                    message.peer_id,
                                    &format!("\u{274C} Failed to download document: {}", e)
                                ).await?;
                            }
                        }
                    }
                }

                "audio" => {
                    if let Some(audio) = &attachment.audio {
                        api.send_message(
                            message.peer_id,
                            &format!("\u{1F3B5} Received audio: {} - {}",
                                audio.artist, audio.title)
                        ).await?;

                        if audio.url.is_some() {
                            match api.download_audio(audio).await {
                                Ok(downloaded) => {
                                    let size_kb = downloaded.data.len() / 1024;
                                    api.send_message(
                                        message.peer_id,
                                        &format!("\u{2705} Audio downloaded! Size: {} KB", size_kb)
                                    ).await?;
                                }
                                Err(e) => {
                                    api.send_message(
                                        message.peer_id,
                                        &format!("\u{274C} Failed to download audio: {}", e)
                                    ).await?;
                                }
                            }
                        } else {
                            api.send_message(
                                message.peer_id,
                                "\u{26A0} Audio URL not available (may be restricted)"
                            ).await?;
                        }
                    }
                }

                "audio_message" => {
                    if let Some(audio_msg) = &attachment.audio_message {
                        api.send_message(
                            message.peer_id,
                            &format!("\u{1F3A4} Received voice message ({} sec)",
                                audio_msg.duration)
                        ).await?;

                        match api.download_audio_message(audio_msg).await {
                            Ok(downloaded) => {
                                let size_kb = downloaded.data.len() / 1024;
                                api.send_message(
                                    message.peer_id,
                                    &format!("\u{2705} Voice message downloaded! Size: {} KB", size_kb)
                                ).await?;
                            }
                            Err(e) => {
                                api.send_message(
                                    message.peer_id,
                                    &format!("\u{274C} Failed to download voice message: {}", e)
                                ).await?;
                            }
                        }
                    }
                }

                "video" => {
                    if let Some(video) = &attachment.video {
                        api.send_message(
                            message.peer_id,
                            &format!("\u{1F3AC} Received video: {}\nDuration: {} sec",
                                video.title, video.duration)
                        ).await?;

                        // Download thumbnail
                        match api.download_video_thumbnail(video).await {
                            Ok(downloaded) => {
                                let size_kb = downloaded.data.len() / 1024;
                                api.send_message(
                                    message.peer_id,
                                    &format!("\u{2705} Video thumbnail downloaded! Size: {} KB", size_kb)
                                ).await?;
                            }
                            Err(e) => {
                                api.send_message(
                                    message.peer_id,
                                    &format!("\u{274C} Failed to download thumbnail: {}", e)
                                ).await?;
                            }
                        }
                    }
                }

                "sticker" => {
                    if let Some(sticker) = &attachment.sticker {
                        api.send_message(
                            message.peer_id,
                            &format!("\u{1F48C} Received sticker #{}!",
                                sticker.sticker_id)
                        ).await?;

                        match api.download_sticker(sticker).await {
                            Ok(downloaded) => {
                                let size_kb = downloaded.data.len() / 1024;
                                api.send_message(
                                    message.peer_id,
                                    &format!("\u{2705} Sticker downloaded! Size: {} KB", size_kb)
                                ).await?;
                            }
                            Err(e) => {
                                api.send_message(
                                    message.peer_id,
                                    &format!("\u{274C} Failed to download sticker: {}", e)
                                ).await?;
                            }
                        }
                    }
                }

                other => {
                    api.send_message(
                        message.peer_id,
                        &format!("\u{1F4E5} Received attachment type: {}", other)
                    ).await?;
                }
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logger
    env_logger::init();

    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Get token and group ID from environment
    let token = std::env::var("VK_TOKEN").expect("VK_TOKEN must be set");
    let group_id = std::env::var("VK_GROUP_ID")
        .expect("VK_GROUP_ID must be set")
        .parse::<i64>()
        .expect("VK_GROUP_ID must be a number");

    println!("\u{1F916} Starting Media Bot for group {}...", group_id);
    println!("\u{1F4E1} Features: Upload & Download");
    println!("");
    println!("Available commands:");
    println!("  /help   - Show help");
    println!("  /photo  - Send example photo");
    println!("  /doc    - Send example document");
    println!("  /voice  - Voice message info");
    println!("  /stats  - Bot statistics");
    println!("");
    println!("Send any attachment to see download in action!");
    println!("");

    // Create bot with custom configuration
    let mut bot = VkBot::builder()
        .token(&token)
        .group_id(group_id)
        .long_poll_timeout(25)
        .auto_reconnect(true)
        .max_concurrent_handlers(20)
        .log_updates(false)  // Set to true for debugging
        .build()?;

    // Add media handler
    bot.add_handler(MediaBotHandler::default());

    // Run bot
    println!("\u{23F3} Bot is running. Press Ctrl+C to stop.");
    println!("");

    bot.run().await?;

    Ok(())
}
