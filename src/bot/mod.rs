use crate::log;
use crate::parser::{Match, Parser};
use crate::telegram::{ParsingCommand, Telegram};
use chrono::prelude::*;
use std::sync::Arc;
use tokio::sync::watch;
use tokio::time::{Duration, interval, sleep};

pub enum TicketsBotStatus {
    Stopped,
    Active,
    Error,
}

pub struct TicketsBot {
    status: TicketsBotStatus,
}

impl TicketsBot {
    pub fn new() -> Self {
        TicketsBot {
            status: TicketsBotStatus::Stopped,
        }
    }

    pub async fn start(&mut self) {
        self.status = TicketsBotStatus::Active;
        log::info("TicketsBot started.");

        if let Err(err) = self.manage_bot().await {
            self.status = TicketsBotStatus::Error;
            log::error(format!("TicketsBot stopped with error: {err}"));
        }
    }

    #[allow(unused)]
    pub fn stop(&mut self) {
        self.status = TicketsBotStatus::Stopped;
        log::info("TicketsBot stopped.");
    }

    #[allow(unused)]
    pub fn get_status(&self) -> &TicketsBotStatus {
        &self.status
    }

    async fn manage_bot(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let url = if let Some(url) = std::env::args().nth(1) {
            url
        } else {
            log::warn("No CLI URL provided, using default.");
            "https://billetterie.staderochelais.com/fr".into()
        };

        let telegram = Arc::new(Telegram::new());
        let parser = Arc::new(Parser::new(url));

        let (parsing_tx, mut parsing_rx) = watch::channel(ParsingCommand::Start);
        Telegram::set_parsing_control(parsing_tx);

        // 1. Telegram commands task
        let telegram_for_commands = Arc::clone(&telegram);
        let mut telegram_commands_task = tokio::spawn(async move {
            log::info("Bot Telegram demarre...");
            Self::check_telegram_commands(&telegram_for_commands).await;
        });

        // 2. Resale parsing task
        let telegram_for_resale = Arc::clone(&telegram);
        let parser_for_resale = Arc::clone(&parser);
        let mut resale_parsing_task = tokio::spawn(async move {
            let mut tick = interval(Duration::from_secs(
                crate::telegram::parsing_interval_seconds(),
            ));
            loop {
                tokio::select! {
                    _ = tick.tick() => {
                        if crate::telegram::parsing_is_running() {
                            Self::check_and_notifify_resale(&parser_for_resale, &telegram_for_resale).await;
                        }
                    }
                    Ok(()) = parsing_rx.changed() => {
                        match *parsing_rx.borrow_and_update() {
                            ParsingCommand::Start => Telegram::set_parsing_running(true),
                            ParsingCommand::Stop => Telegram::set_parsing_running(false),
                            ParsingCommand::SetInterval(seconds) => {
                                tick = interval(Duration::from_secs(seconds));
                                Telegram::set_parsing_interval_seconds(seconds);
                                log::info(format!(
                                    "Resale parsing interval updated to {} second(s).",
                                    seconds
                                ));
                            }
                        }
                    }
                }
            }
        });

        // 3. Calendar parsing task
        let parser_for_calendar = Arc::clone(&parser);
        let telegram_for_calendar = Arc::clone(&telegram);
        let mut calendar_parsing_task = tokio::spawn(async move {
            let mut last_weekly_notification_day: Option<NaiveDate> = None;
            loop {
                let next_check_duration = Self::check_and_notifify_calendar(
                    &parser_for_calendar,
                    &telegram_for_calendar,
                    &mut last_weekly_notification_day,
                )
                .await;
                sleep(Duration::from_secs(next_check_duration)).await;
            }
        });

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                log::warn("Ctrl+C recu, arret du bot...");
                self.status = TicketsBotStatus::Stopped;
            }

            result = &mut telegram_commands_task => {
                match result {
                    Ok(_) => {
                        log::info("La tache Telegram s'est arretee.");
                        self.status = TicketsBotStatus::Stopped;
                    }
                    Err(err) => {
                        log::error(format!("La tache Telegram a echoue: {err}"));
                        self.status = TicketsBotStatus::Error;
                    }
                }
            }

            result = &mut resale_parsing_task => {
                match result {
                    Ok(_) => {
                        log::info("La tache de parsing resale s'est arretee.");
                        self.status = TicketsBotStatus::Stopped;
                    }
                    Err(err) => {
                        log::error(format!("La tache de parsing resale a echoue: {err}"));
                        self.status = TicketsBotStatus::Error;
                    }
                }
            }

            result = &mut calendar_parsing_task => {
                match result {
                    Ok(_) => {
                        log::info("La tache de parsing calendar s'est arretee.");
                        self.status = TicketsBotStatus::Stopped;
                    }
                    Err(err) => {
                        log::error(format!("La tache de parsing calendar a echoue: {err}"));
                        self.status = TicketsBotStatus::Error;
                    }
                }
            }
        }

        if !telegram_commands_task.is_finished() {
            telegram_commands_task.abort();
            let _ = telegram_commands_task.await;
        }

        if !resale_parsing_task.is_finished() {
            resale_parsing_task.abort();
            let _ = resale_parsing_task.await;
        }

        if !calendar_parsing_task.is_finished() {
            calendar_parsing_task.abort();
            let _ = calendar_parsing_task.await;
        }

        Ok(())
    }

    async fn check_and_notifify_resale(parser: &Parser, telegram: &Telegram) {
        let match_to_be_resaled: Vec<Match> = Vec::new();

        let matches = match parser.fetch_and_parse().await {
            Ok(matches) => matches,
            Err(err) => {
                log::error(format!("Error while fetching/parsing ticket page: {err}"));
                return;
            }
        };

        let new_matches: Vec<_> = matches
            .iter()
            .filter(|m| !match_to_be_resaled.iter().any(|h| h.title == m.title))
            .cloned()
            .collect();

        if !new_matches.is_empty() {
            log::info(format!("Found {} new resale match(es)", new_matches.len()));

            telegram
                .notify_telegram(&new_matches)
                .await
                .unwrap_or_else(|err| {
                    log::error(format!("Error sending Telegram notification: {err}"))
                });
        } else if !matches.is_empty() {
            log::info("No new resale matches found");
        } else {
            log::info("No resale matches found");
        }

        for removed in match_to_be_resaled
            .iter()
            .filter(|h| !matches.iter().any(|m| m.title == h.title))
        {
            log::info(format!(
                "Match {} is no longer available for resale.",
                removed.title
            ));
        }
    }

    async fn check_telegram_commands(telegram: &Telegram) {
        telegram.run_commands().await;
    }

    async fn check_and_notifify_calendar(
        parser: &Parser,
        telegram: &Telegram,
        last_weekly_notification_day: &mut Option<NaiveDate>,
    ) -> u64 {
        let next_check_duration = 3600;

        // Step 1 : Get the next upcoming match by parsing the URLs
        let now = Utc::now();
        let next_match = parser.next_upcoming_match_from_urls(now).await;

        let Some(next_match) = next_match else {
            log::info("No upcoming matches found in the calendar.");
            return next_check_duration;
        };

        let Some(next_match_time) = next_match.timestamp else {
            log::warn("Next match has no timestamp, skipping notification rules.");
            return next_check_duration;
        };

        let time_until_next_match = next_match_time - now;
        log::info(format!(
            "Next match '{}' is in {} seconds",
            next_match.match_title,
            time_until_next_match.num_seconds()
        ));

        if Self::is_necessity_to_notify_calendar(time_until_next_match, now) {
            if let Err(err) = telegram
                .notify_imminent_match(time_until_next_match.num_minutes())
                .await
            {
                log::error(format!(
                    "Error sending Telegram imminent match notification: {err}"
                ));
            } else {
                log::info("Telegram imminent match notification sent successfully.");
            }
            return next_check_duration;
        }

        if Self::is_weekly_calendar_notification_window(
            time_until_next_match,
            now,
            last_weekly_notification_day,
        ) {
            log::info(
                "It's Monday or Friday and the next match is in less than 7 days, sending notification...",
            );
            if let Err(err) = telegram.notify_calendar(&next_match).await {
                log::error(format!(
                    "Error sending Telegram calendar notification: {err}"
                ));
            } else {
                log::info("Telegram calendar notification sent successfully.");
                *last_weekly_notification_day = Some(now.date_naive());
            }
        } else {
            log::info(
                "No calendar notification sent (outside Monday/Friday window, already sent today, or match is too far away).",
            );
        }
        next_check_duration
    }

    fn is_necessity_to_notify_calendar(
        time_until_next_match: chrono::Duration,
        _now: DateTime<Utc>,
    ) -> bool {
        // If the match is in less than 2 hours, we alert users immediately.
        time_until_next_match.num_seconds() < 2 * 3600
    }

    fn is_weekly_calendar_notification_window(
        time_until_next_match: chrono::Duration,
        now: DateTime<Utc>,
        last_weekly_notification_day: &Option<NaiveDate>,
    ) -> bool {
        if last_weekly_notification_day == &Some(now.date_naive()) {
            return false;
        }

        if ((now.weekday() == Weekday::Mon) || (now.weekday() == Weekday::Fri))
            && time_until_next_match.num_seconds() < 7 * 24 * 3600
        {
            // check if this is 8 a.m to avoid sending notification too early in the morning
            return now.hour() >= 8;
        }
        false
    }
}
