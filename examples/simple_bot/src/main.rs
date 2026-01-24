use std::error::Error;
use vk_bot_api::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logger
    env_logger::init();

    // Load environment variables
    dotenv::dotenv().ok();

    let token = std::env::var("VK_TOKEN").expect("VK_TOKEN must be set");

    let group_id = std::env::var("VK_GROUP_ID")
        .expect("VK_GROUP_ID must be set")
        .parse::<i64>()
        .expect("VK_GROUP_ID must be a number");

    println!("Starting bot for group {}...", group_id);

    // Create bot with custom configuration
    let mut bot = VkBot::builder()
        .token(&token)
        .group_id(group_id)
        .long_poll_timeout(25)
        .auto_reconnect(true)
        .max_concurrent_handlers(10)
        .log_updates(true)
        .build()?;

    // Add default handler
    bot.add_handler(DefaultMessageHandler);

    // Add admin handler if admin IDs are specified
    if let Ok(admin_ids_str) = std::env::var("VK_ADMIN_IDS") {
        let admin_ids: Vec<i64> = admin_ids_str
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();

        if !admin_ids.is_empty() {
            bot.add_handler(AdminHandler::new(admin_ids.clone()));
            println!("Admin handler added for IDs: {:?}", admin_ids);
        }
    }

    // Run bot
    bot.run().await?;

    Ok(())
}
