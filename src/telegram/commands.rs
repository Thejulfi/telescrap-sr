use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

use super::ParsingCommand;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub(super) enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "Launch the bot")]
    Start,
    #[command(description = "Stop the bot")]
    Stop,
    #[command(description = "Get the current status of the bot")]
    Status,
    #[command(
        description = "Set new parsing polling interval in seconds. Usage: \"/setinterval <seconds>\""
    )]
    SetInterval(u64),
    #[command(description = "Export logs")]
    GetLog,
}

pub(super) async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    let admin_ids = parse_admin_chat_ids().unwrap_or_default();

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
            let parsing_status = if super::parsing_is_running() {
                "active"
            } else {
                "stopped"
            };
            let interval_seconds = super::parsing_interval_seconds();

            bot.send_message(
                msg.chat.id,
                format!(
                    "Parsing is currently {}. Polling interval: {} second(s).",
                    parsing_status, interval_seconds
                ),
            )
            .await?
        }
        Command::Start => {
            if super::send_parsing_command(ParsingCommand::Start) {
                bot.send_message(msg.chat.id, "Parsing started.").await?
            } else {
                bot.send_message(msg.chat.id, "Unable to start parsing right now.")
                    .await?
            }
        }
        Command::Stop => {
            if super::send_parsing_command(ParsingCommand::Stop) {
                bot.send_message(msg.chat.id, "Parsing stopped.").await?
            } else {
                bot.send_message(msg.chat.id, "Unable to stop parsing right now.")
                    .await?
            }
        }
        Command::SetInterval(seconds) => {
            if seconds == 0 {
                bot.send_message(msg.chat.id, "Interval must be greater than 0 second.")
                    .await?
            } else if super::send_parsing_command(ParsingCommand::SetInterval(seconds)) {
                bot.send_message(
                    msg.chat.id,
                    format!("Parsing interval set to {} second(s).", seconds),
                )
                .await?
            } else {
                bot.send_message(msg.chat.id, "Unable to set parsing interval right now.")
                    .await?
            }
        }
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
