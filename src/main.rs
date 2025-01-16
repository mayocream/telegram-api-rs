use telegram_api::{Bot, SendMessageRequest, TelegramError};

// Example usage:
#[tokio::main]
async fn main() -> Result<(), TelegramError> {
    let bot = Bot::new("YOUR_BOT_TOKEN".to_string())?;

    // Get updates
    let updates = bot.get_updates(None).await?;
    for update in updates {
        if let Some(message) = update.message {
            if let Some(text) = message.text {
                // Echo the message back
                let request = SendMessageRequest {
                    chat_id: message.chat.id,
                    text: text,
                    reply_to_message_id: Some(message.message_id),
                    parse_mode: None,
                };
                bot.send_message(request).await?;
            }
        }
    }

    Ok(())
}
