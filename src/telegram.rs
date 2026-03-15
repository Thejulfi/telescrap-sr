use crate::parser;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "Launch the bot")]
    Start,
    #[command(description = "Stop the bot")]
    Stop,
    #[command(description = "Get the current status of the bot")]
    Status,
    #[command(
        description = "Set new parsing polling interval in minutes. Usage: \"/setinterval <minutes>\""
    )]
    SetInterval,
    #[command(description = "Export logs")]
    GetLog,
}

#[derive(Clone)]
pub struct Telegram {
    bot: Bot,
    notifier_id: i64,
}

impl Telegram {
    pub async fn new() -> Self {
        dotenvy::dotenv().ok();

        let notifier_id = Self::parse_chat_id_from_env("TELEGRAM_CHAT_ID")
            .expect("Invalid or missing TELEGRAM_CHAT_ID in environment");

        Command::repl(Bot::from_env(), Self::answer).await;

        Telegram {
            bot: Bot::from_env(),
            notifier_id,
        }
    }

    pub async fn notify_telegram(
        &self,
        matches: &[parser::Match],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut message = String::from("Reventes disponibles:\n");
        for m in matches {
            message.push_str(&format!(
                "- {}\nLink : {}\n",
                m.title,
                m.url.as_deref().unwrap_or("N/A")
            ));
        }

        self.bot
            .send_message(ChatId(self.notifier_id), message)
            .await?;
        Ok(())
    }

    async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
        let admin_ids = Self::parse_admin_chat_ids().unwrap_or_default();

        if !admin_ids.contains(&msg.chat.id.0) {
            bot.send_message(msg.chat.id, "Unauthorized access.")
                .await?;
            return Ok(());
        }

        match cmd {
            Command::Help => {
                bot.send_message(msg.chat.id, Command::descriptions().to_string())
                    .await?
            }
            Command::Status => {
                bot.send_message(msg.chat.id, "Here is the bot status: STATUS")
                    .await?
            }
            Command::Start => bot.send_message(msg.chat.id, "Bot started!").await?,
            Command::Stop => bot.send_message(msg.chat.id, "Bot stopped!").await?,
            Command::SetInterval => bot.send_message(msg.chat.id, "Interval set!").await?,
            Command::GetLog => bot.send_message(msg.chat.id, "Logs exported!").await?,
        };

        Ok(())
    }

    fn parse_admin_chat_ids() -> Result<Vec<i64>, Box<dyn std::error::Error>> {
        let raw = std::env::var("TELEGRAM_ADMIN_CHAT_IDS").unwrap_or_default();

        if raw.trim().is_empty() {
            return Err("TELEGRAM_ADMIN_CHAT_IDS is not set or empty".into());
        }

        raw.split(',')
            .map(|value| value.trim().parse::<i64>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| err.into())
    }

    fn parse_chat_id_from_env(var_name: &str) -> Result<i64, Box<dyn std::error::Error>> {
        Ok(std::env::var(var_name)?.parse()?)
    }
}
