use scanner::controller::notify::Notify;
use teloxide::{
    prelude::*,
    types::ParseMode,
};

/// This module defines the `TelegramNotifier` struct, which implements the `Notify` trait to send notifications to a Telegram chat.
pub struct TelegramNotifier {
    bot_token: String,
    chat_id: i64,
}

impl TelegramNotifier {
    /// Creates a new instance of `TelegramNotifier` with the specified bot token and chat ID.
    pub fn new(bot_token: String, chat_id: i64) -> Self {
        Self { bot_token, chat_id }
    }
}

impl Notify for TelegramNotifier {
    /// Sends a message to the configured Telegram chat using the bot token and chat ID.
    /// This method creates a new `Bot` instance with the provided bot token, constructs a `ChatId` from the chat ID,
    /// and sends the message using the Telegram Bot API.
    /// 
    /// # Arguments
    /// * `message` - A string slice containing the message to be sent to the Telegram chat.
    /// The message can include HTML formatting, which will be parsed by Telegram when displayed.
    /// 
    /// # Returns
    /// This method does not return a value, but it sends the specified message to the Telegram chat associated with the provided chat ID using the bot token for authentication.
    fn send(&self, message: &str) {
        let bot = Bot::new(&self.bot_token);
        let chat_id = ChatId(self.chat_id);
        let message = message.to_string();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                bot.send_message(chat_id, message).parse_mode(ParseMode::Html).await.ok();
            });
        });
    }
}
