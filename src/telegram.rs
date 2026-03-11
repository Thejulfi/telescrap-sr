use crate::parser;
use teloxide::prelude::*;
use teloxide::types::{ChatId, UpdateKind};
use tokio::time::{self, Duration};

const COMMANDS_HELP: &str = "
Commandes disponibles:\n
/status - Etat du bot\n/help - Affiche cette aide";

const COMMAND_POLL_TIMEOUT_SECONDS: u32 = 5;
const COMMAND_POLL_IDLE_DELAY_MS: u64 = 700;

#[derive(Clone)]
pub struct Telegram {
    bot: Bot,
    notifier_id: i64,
    admin_id: Vec<i64>,
}

impl Telegram {
    pub async fn new() -> Self {
        dotenvy::dotenv().ok();

        let notifier_id = Self::parse_chat_id_from_env("TELEGRAM_CHAT_ID")
            .expect("Invalid or missing TELEGRAM_CHAT_ID in environment");

        let admin_id =
            Self::parse_admin_chat_ids(Self::parse_chat_id_from_env("TELEGRAM_CHAT_ID").unwrap())
                .expect(
                    "Invalid TELEGRAM_ADMIN_CHAT_IDS format. Expected comma-separated chat IDs",
                );

        Telegram {
            bot: Bot::from_env(),
            notifier_id,
            admin_id,
        }
    }

    pub async fn notify_telegram(
        &self,
        matches: &[parser::Match],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut message = String::from("Reventes disponibles:\n");
        for m in matches {
            message.push_str(&format!("- {}\n", m.title));
        }

        self.bot
            .send_message(ChatId(self.notifier_id), message)
            .await?;
        Ok(())
    }

    pub async fn run_command_listener(&self) {
        let mut next_offset: i32 = 0;

        loop {
            match self
                .bot
                .get_updates()
                .offset(next_offset)
                .timeout(COMMAND_POLL_TIMEOUT_SECONDS)
                .send()
                .await
            {
                Ok(updates) => {
                    for update in updates {
                        next_offset = update.id.0 as i32 + 1;

                        if let UpdateKind::Message(message) = update.kind {
                            Self::handle_command(&self.bot, &message, &self.admin_id).await;
                        }
                    }

                    // Short-polling is often more reliable behind proxies/devcontainers.
                    time::sleep(Duration::from_millis(COMMAND_POLL_IDLE_DELAY_MS)).await;
                }
                Err(err) => {
                    eprintln!("Command listener error: {err}");
                    time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }

    async fn handle_command(bot: &Bot, msg: &Message, admin_chat_ids: &[i64]) {
        let Some(text) = msg.text() else {
            return;
        };

        if !text.starts_with('/') {
            return;
        }

        let chat_id = msg.chat.id.0;
        if !admin_chat_ids.contains(&chat_id) {
            let _ = bot
                .send_message(msg.chat.id, "Ce chat n'est pas autorise a piloter le bot.")
                .await;
            return;
        }

        let command = text
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .split('@')
            .next()
            .unwrap_or_default();

        let response = match command {
            "/status" => Some(format!(
                "Bot actif. Scraping en cours toutes les {} minutes.",
                1
            )),
            "/help" | "/start" => Some(COMMANDS_HELP.to_string()),
            _ => Some("Commande inconnue. Utilise /help".to_string()),
        };

        if let Some(reply) = response {
            let _ = bot.send_message(msg.chat.id, reply).await;
        }
    }

    fn parse_admin_chat_ids(default_chat_id: i64) -> Result<Vec<i64>, Box<dyn std::error::Error>> {
        let raw = std::env::var("TELEGRAM_ADMIN_CHAT_IDS").unwrap_or_default();
        if raw.trim().is_empty() {
            return Ok(vec![default_chat_id]);
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
