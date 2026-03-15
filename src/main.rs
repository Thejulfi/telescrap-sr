mod parser;
mod telegram;
use chrono::Local;
use parser::Match;
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
    let parser = Parser::new(url);

    let mut match_to_be_resaled: Vec<Match> = Vec::new();

    let mut ticker = time::interval(Duration::from_secs(PARSING_WEB_TIMEOUT_SECONDS * 60));

    loop {
        // attend le prochain "tick"
        ticker.tick().await;

        let matches = parser.fetch_and_parse().await?;

        let new_matches: Vec<_> = matches
            .iter()
            .filter(|m| !match_to_be_resaled.iter().any(|h| h.title == m.title))
            .cloned()
            .collect();

        if !new_matches.is_empty() {
            let ts = Local::now().format("%Y-%m-%d %H:%M:%S");
            println!("[{}] Found {} new resale match(es)", ts, new_matches.len());

            Telegram::notify_telegram(&telegram, &new_matches)
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

        // Keep a snapshot of current resale matches to compute diffs on next tick.
        match_to_be_resaled = matches.clone();
    }
}
// The [cfg(not(target_arch = "wasm32"))] above prevent building the tokio::main function
// for wasm32 target, because tokio isn't compatible with wasm32.
// If you aren't building for wasm32, you don't need that line.
// The two lines below avoid the "'main' function not found" error when building for wasm32 target.
#[cfg(target_arch = "wasm32")]
fn main() {}
