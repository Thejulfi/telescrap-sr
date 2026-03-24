mod commands;

use crate::parser;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use teloxide::prelude::*;
use tokio::sync::watch;

#[derive(Clone, Copy, Debug)]
pub enum ParsingCommand {
    Start,
    Stop,
    SetInterval(u64),
}

static PARSING_CONTROL_TX: Mutex<Option<watch::Sender<ParsingCommand>>> = Mutex::new(None);
static PARSING_RUNNING: AtomicBool = AtomicBool::new(true);
const DEFAULT_PARSING_INTERVAL_SECONDS: u64 = 60;
static PARSING_INTERVAL_SECONDS: AtomicU64 = AtomicU64::new(DEFAULT_PARSING_INTERVAL_SECONDS);

static SENT_MESSAGES: Mutex<VecDeque<teloxide::types::MessageId>> = Mutex::new(VecDeque::new());
const MAX_TRACKED: usize = 500;

pub(in crate::telegram) fn send_parsing_command(command: ParsingCommand) -> bool {
    if let Ok(guard) = PARSING_CONTROL_TX.lock()
        && let Some(tx) = guard.as_ref()
    {
        return tx.send(command).is_ok();
    }
    false
}

pub fn parsing_is_running() -> bool {
    PARSING_RUNNING.load(Ordering::Relaxed)
}

pub fn parsing_interval_seconds() -> u64 {
    PARSING_INTERVAL_SECONDS.load(Ordering::Relaxed)
}

pub fn notifier_chat_id_from_env() -> Option<ChatId> {
    std::env::var("TELEGRAM_CHAT_ID")
        .ok()
        .and_then(|raw| raw.parse::<i64>().ok())
        .map(ChatId)
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
        self.send_and_track(message).await?;
        Ok(())
    }

    pub async fn notify_calendar(
        &self,
        next_match: &parser::MatchDetails,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut message = String::from("<b>Prochain match :</b>\n\n");
        message.push_str(&format!(
            "🏉 {}\n🏆 Championnat : {}\n📆 : {}\n🕒 : {}\n\n",
            next_match.match_title,
            next_match.championship,
            next_match.date_human_readable,
            next_match
                .hour
                .map(|h| h.format("%H:%M").to_string())
                .unwrap_or_else(|| "Heure non définie".to_string())
        ));
        self.send_and_track(message).await?;
        Ok(())
    }

    pub async fn notify_imminent_match(
        &self,
        minutes_until_match: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut message = String::from("<b>Le match de La Rochelle va bientôt commencer</b>\n\n");
        message.push_str(&format!("🕒 Dans... {} minutes\n\n", minutes_until_match));
        self.send_and_track(message).await?;
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

    pub fn set_parsing_interval_seconds(seconds: u64) {
        PARSING_INTERVAL_SECONDS.store(seconds, Ordering::Relaxed);
    }

    fn parse_chat_id_from_env(var_name: &str) -> Result<i64, Box<dyn std::error::Error>> {
        Ok(std::env::var(var_name)?.parse()?)
    }

    async fn send_and_track(&self, text: String) -> ResponseResult<()> {
        let sent = self
            .bot
            .send_message(ChatId(self.notifier_id), text)
            .parse_mode(teloxide::types::ParseMode::Html)
            .await?;
        if let Ok(mut q) = SENT_MESSAGES.lock() {
            q.push_back(sent.id);
            while q.len() > MAX_TRACKED {
                q.pop_front();
            }
        }
        Ok(())
    }
}

pub async fn clear_last_in_chat(bot: &Bot, chat_id: ChatId, n: usize) -> ResponseResult<usize> {
    let mut ids = Vec::new();
    if let Ok(mut q) = SENT_MESSAGES.lock() {
        for _ in 0..n {
            if let Some(id) = q.pop_back() {
                ids.push(id);
            } else {
                break;
            }
        }
    }

    let mut deleted = 0usize;
    for id in ids {
        if bot.delete_message(chat_id, id).await.is_ok() {
            deleted += 1;
        }
    }
    Ok(deleted)
}
