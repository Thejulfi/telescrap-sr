use crate::parser::{Match, Parser};
use crate::telegram::{ParsingCommand, Telegram};
use chrono::Local;
use tokio::sync::watch;
use tokio::time::{self, Duration};

const PARSING_WEB_TIMEOUT_SECONDS: u64 = 1;

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
        println!("TicketsBot started.");

        if let Err(err) = self.manage_bot().await {
            self.status = TicketsBotStatus::Error;
            eprintln!("TicketsBot stopped with error: {err}");
        }
    }

    #[allow(unused)]
    pub fn stop(&mut self) {
        self.status = TicketsBotStatus::Stopped;
        println!("TicketsBot stopped.");
    }

    #[allow(unused)]
    pub fn get_status(&self) -> &TicketsBotStatus {
        &self.status
    }

    async fn manage_bot(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let url = if let Some(url) = std::env::args().nth(1) {
            url
        } else {
            println!("No CLI URL provided, using default.");
            "https://billetterie.staderochelais.com/fr".into()
        };

        let telegram = Telegram::new();
        let parser = Parser::new(url);
        let (parsing_tx, mut parsing_rx) = watch::channel(ParsingCommand::Start);
        Telegram::set_parsing_control(parsing_tx);
        Telegram::set_parsing_running(true);

        let mut parsing_enabled = true;
        let telegram_commands = telegram.clone();
        let mut telegram_commands_task = tokio::spawn(async move {
            telegram_commands.run_commands().await;
        });

        let mut match_to_be_resaled: Vec<Match> = Vec::new();

        let mut ticker = time::interval(Duration::from_secs(PARSING_WEB_TIMEOUT_SECONDS * 60));

        loop {
            let mut should_parse = false;

            tokio::select! {
                result = &mut telegram_commands_task => {
                    if let Err(err) = result {
                        eprintln!("Telegram commands task failed: {err}");
                        self.status = TicketsBotStatus::Error;
                    } else {
                        println!("Telegram commands task stopped, shutting down bot.");
                        self.status = TicketsBotStatus::Stopped;
                    }
                    break;
                }
                changed = parsing_rx.changed() => {
                    if changed.is_ok() {
                        match *parsing_rx.borrow_and_update() {
                            ParsingCommand::Start => {
                                if !parsing_enabled {
                                    println!("Parsing resumed.");
                                }
                                parsing_enabled = true;
                                Telegram::set_parsing_running(true);
                            }
                            ParsingCommand::Stop => {
                                if parsing_enabled {
                                    println!("Parsing paused.");
                                }
                                parsing_enabled = false;
                                Telegram::set_parsing_running(false);
                            }
                        }
                    }
                }
                _ = ticker.tick(), if parsing_enabled => {
                    should_parse = true;
                }
            }

            if !should_parse {
                continue;
            }

            let matches = match parser.fetch_and_parse().await {
                Ok(matches) => matches,
                Err(err) => {
                    eprintln!("Error while fetching/parsing ticket page: {err}");
                    continue;
                }
            };

            let new_matches: Vec<_> = matches
                .iter()
                .filter(|m| !match_to_be_resaled.iter().any(|h| h.title == m.title))
                .cloned()
                .collect();

            if !new_matches.is_empty() {
                let ts = Local::now().format("%Y-%m-%d %H:%M:%S");
                println!("[{}] Found {} new resale match(es)", ts, new_matches.len());

                telegram
                    .notify_telegram(&new_matches)
                    .await
                    .unwrap_or_else(|err| eprintln!("Error sending Telegram notification: {err}"));
            } else if !matches.is_empty() {
                println!("No new resale matches found");
            } else {
                println!("No resale matches found");
            }

            for removed in match_to_be_resaled
                .iter()
                .filter(|h| !matches.iter().any(|m| m.title == h.title))
            {
                println!("Match {} is no longer available for resale.", removed.title);
            }

            match_to_be_resaled = matches.clone();
        }

        Telegram::set_parsing_running(false);

        if !telegram_commands_task.is_finished() {
            telegram_commands_task.abort();
        }

        Ok(())
    }
}
