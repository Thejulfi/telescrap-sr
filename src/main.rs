mod parser;
mod telegram;
use chrono::Local;
use parser::Parser;
use telegram::Telegram;
use tokio::time::{self, Duration};

const PARSING_WEB_TIMEOUT_SECONDS: u64 = 1;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    // Some simple CLI args requirements...
    let url = if let Some(url) = std::env::args().nth(1) {
        url
    } else {
        println!("No CLI URL provided, using default.");
        "https://billetterie.staderochelais.com/fr".into()
    };

    let telegram = Telegram::new().await;

    let telegram_for_commands = telegram.clone();
    tokio::spawn(async move {
        telegram_for_commands.run_command_listener().await;
    });

    let parser = Parser::new(url);

    let mut ticker = time::interval(Duration::from_secs(PARSING_WEB_TIMEOUT_SECONDS * 60));

    loop {
        // attend le prochain "tick"
        ticker.tick().await;

        let matches = parser.fetch_and_parse().await?;

        let ts = Local::now().format("%Y-%m-%d %H:%M:%S");
        println!("[{}] Found {} matches:", ts, matches.len());

        if !matches.is_empty() {
            Telegram::notify_telegram(&telegram, &matches)
                .await
                .unwrap_or_else(|err| eprintln!("Error sending Telegram notification: {err}"));
        }
    }
}
// The [cfg(not(target_arch = "wasm32"))] above prevent building the tokio::main function
// for wasm32 target, because tokio isn't compatible with wasm32.
// If you aren't building for wasm32, you don't need that line.
// The two lines below avoid the "'main' function not found" error when building for wasm32 target.
#[cfg(target_arch = "wasm32")]
fn main() {}
