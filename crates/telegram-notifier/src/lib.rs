use scanner::controller::notify::Notify;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

pub struct TelegramNotifier {
    bot_token: String,
    chat_id: i64,
}

impl TelegramNotifier {
    pub fn new(bot_token: String, chat_id: i64) -> Self {
        Self { bot_token, chat_id }
    }
}

impl Notify for TelegramNotifier {
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
