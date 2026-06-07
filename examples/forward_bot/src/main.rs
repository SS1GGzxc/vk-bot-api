use async_trait::async_trait;
use std::error::Error;
use vk_bot_api::prelude::*;

#[derive(Debug, Clone, Default)]
struct ForwardMessageHandler;

#[async_trait]
impl MessageHandler for ForwardMessageHandler {
    async fn handle(&self, event: &Event, api: &VkApi) -> vk_bot_api::error::VkResult<()> {
        if let Event::MessageNew(message) = event {
            // Mark message as read
            let _ = api
                .messages_mark_as_read(message.peer_id, Some(message.id))
                .await;

            let command = message.text.split_whitespace().next().unwrap_or("");

            match command {
                "/forward" => {
                    // Send a message and forward the original user's message
                    let forward_ids = [message.id];
                    api.messages_send(
                        message.peer_id,
                        "This message forwards your previous message:",
                        None,
                        None,
                        None,
                        None,
                        Some(&forward_ids),
                        false,
                        false,
                        None,
                    )
                    .await?;
                }

                "/reply" => {
                    // Reply to the original message
                    api.messages_send(
                        message.peer_id,
                        "This is a direct reply to your message!",
                        None,
                        None,
                        None,
                        Some(message.id), // Use reply_to parameter
                        None,
                        false,
                        false,
                        None,
                    )
                    .await?;
                }

                "/help" | "/start" => {
                    api.messages_send(
                        message.peer_id,
                        "Available commands:\n/forward - Bot forwards your message back to you\n/reply - Bot replies to your message",
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

                _ => {}
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    dotenv::dotenv().ok();

    let token = std::env::var("VK_TOKEN").expect("VK_TOKEN must be set");
    let group_id = std::env::var("VK_GROUP_ID")
        .expect("VK_GROUP_ID must be set")
        .parse::<i64>()
        .expect("VK_GROUP_ID must be a number");

    println!("Starting forward_bot for group {}...", group_id);

    let mut bot = VkBot::builder().token(&token).group_id(group_id).build()?;

    bot.add_handler(ForwardMessageHandler);

    println!("Bot started! Send /forward or /reply to test message forwarding.");
    bot.run().await?;

    Ok(())
}
