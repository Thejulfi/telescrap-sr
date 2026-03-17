mod commands;

use crate::parser;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use teloxide::prelude::*;
use tokio::sync::watch;

#[derive(Clone, Copy, Debug)]
pub enum ParsingCommand {
    Start,
    Stop,
}

static PARSING_CONTROL_TX: Mutex<Option<watch::Sender<ParsingCommand>>> = Mutex::new(None);
static PARSING_RUNNING: AtomicBool = AtomicBool::new(true);

/// Envoie une commande au contrôleur de parsing.
/// Visible uniquement dans ce module et ses sous-modules.
pub(in crate::telegram) fn send_parsing_command(command: ParsingCommand) -> bool {
    if let Ok(guard) = PARSING_CONTROL_TX.lock()
        && let Some(tx) = guard.as_ref()
    {
        return tx.send(command).is_ok();
    }
    false
}

/// Retourne true si le parsing est actif.
/// Visible uniquement dans ce module et ses sous-modules.
pub(in crate::telegram) fn parsing_is_running() -> bool {
    PARSING_RUNNING.load(Ordering::Relaxed)
}

#[derive(Clone)]
pub struct Telegram {
    bot: Bot,
    notifier_id: i64,
}

impl Telegram {
    pub fn new() -> Self {
        dotenvy::dotenv().ok();

        let notifier_id = Self::parse_chat_id_from_env("TELEGRAM_CHAT_ID")
            .expect("Invalid or missing TELEGRAM_CHAT_ID in environment");

        Telegram {
            bot: Bot::from_env(),
            notifier_id,
        }
    }

    pub async fn notify_telegram(
        &self,
        matches: &[parser::Match],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut message = String::from("Nouvelles reventes disponibles:\n\n");
        for m in matches {
            message.push_str(&format!(
                "🏉 <b>{}</b>\n🔗 <a href=\"{}\">Acheter</a>\n📆 : {}\n\n",
                m.title,
                m.url.as_deref().unwrap_or("#"),
                m.date
            ));
        }

        self.bot
            .send_message(ChatId(self.notifier_id), message)
            .parse_mode(teloxide::types::ParseMode::Html)
            .await?;
        Ok(())
    }

    pub async fn run_commands(&self) {
        commands::Command::repl(self.bot.clone(), commands::answer).await;
    }

    pub fn set_parsing_control(tx: watch::Sender<ParsingCommand>) {
        if let Ok(mut guard) = PARSING_CONTROL_TX.lock() {
            *guard = Some(tx);
        }
    }

    pub fn set_parsing_running(running: bool) {
        PARSING_RUNNING.store(running, Ordering::Relaxed);
    }

    fn parse_chat_id_from_env(var_name: &str) -> Result<i64, Box<dyn std::error::Error>> {
        Ok(std::env::var(var_name)?.parse()?)
    }
}
