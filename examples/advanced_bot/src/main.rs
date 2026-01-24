use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use std::error::Error;
use vk_bot_api::prelude::*;

#[derive(Debug, Clone, Default)]
struct StatsHandler {
    message_count: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

#[async_trait]
impl MessageHandler for StatsHandler {
    async fn handle(&self, event: &Event, api: &VkApi) -> VkResult<()> {
        match event {
            Event::MessageEvent(event) => match &event.payload {
                Some(payload) => {
                    let command = &payload["command"];
                    api.messages_send(
                        event.peer_id,
                        &format!("command: {}", command),
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
                    return Ok(());
                }
                None => {
                    api.messages_send(
                        event.peer_id,
                        &"Failed to get payload",
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
                    return Ok(());
                }
            },
            Event::MessageNew(message) if message.text.starts_with("/stats") => {
                let count = self.message_count.load(std::sync::atomic::Ordering::SeqCst);
                let now = Utc::now();

                let key = Keyboard::new()
                    .add_callback_button("GOIDA", json!({"command": "goidagoida"}), None)
                    .add_row(Vec::new())
                    .add_callback_button("ZALUPA", json!({"command": "zalupazalupa"}), None);

                api.messages_send(
                    message.peer_id,
                    &format!(
                        "📊 Bot Statistics:\n\nMessages processed: {}\nCurrent time: {}",
                        count,
                        now.format("%Y-%m-%d %H:%M:%S")
                    ),
                    Some(&key),
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
            _ => {}
        }

        return Ok(());
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    dotenv::dotenv().ok();

    let token = std::env::var("VK_TOKEN")?;
    let group_id = std::env::var("VK_GROUP_ID")?.parse::<i64>()?;

    println!("🚀 Starting advanced bot...");

    // Create bot with advanced configuration
    let mut bot = VkBot::builder()
        .token(&token)
        .group_id(group_id)
        .max_concurrent_handlers(20)
        .log_updates(true)
        .build()?;

    // Add multiple handlers
    bot.add_handler(DefaultMessageHandler);
    bot.add_handler(StatsHandler::default());
    // bot.add_handler(WeatherHandler {
    //     api_key: std::env::var("WEATHER_API_KEY").unwrap_or_default(),
    // });

    // Add admin handler if specified
    if let Ok(admin_ids) = std::env::var("VK_ADMIN_IDS") {
        let ids: Vec<i64> = admin_ids
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();

        if !ids.is_empty() {
            println!("⚙️ Admin handlers enabled for IDs: {ids:?}");
            bot.add_handler(AdminHandler::new(ids));
        }
    }

    // Get bot info before starting
    let group_info = bot
        .api()
        .groups_get_by_id(&[bot.group_id()], Some("name,description"))
        .await?;
    if let Some(group) = group_info["response"]
        .as_array()
        .and_then(|arr| arr.first())
    {
        let name = group["name"].as_str().unwrap_or("Unknown");
        println!("🤖 Bot initialized for group: {}", name);
    }

    // Run the bot
    println!("⏳ Bot is running. Press Ctrl+C to stop.");
    bot.run().await?;

    Ok(())
}
