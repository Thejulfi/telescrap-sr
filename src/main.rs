mod parser;
use parser::Parser;
use teloxide::prelude::*;
use teloxide::types::ChatId;
use tokio::time::{self, Duration};

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

    let parser = Parser::new(url);

    let mut ticker = time::interval(Duration::from_secs(1 * 60));

    loop {
        // attend le prochain "tick"
        ticker.tick().await;

        let matches = parser.fetch_and_parse().await?;

        println!("Found {} matches:", matches.len());

        if !matches.is_empty() {
            notify_telegram(&matches)
                .await
                .unwrap_or_else(|err| eprintln!("Error sending Telegram notification: {err}"));
        }
        // else {
        //     test_sending()
        //         .await
        //         .unwrap_or_else(|err| eprintln!("Error sending Telegram test message: {err}"));
        // }
    }
}

async fn notify_telegram(matches: &[parser::Match]) -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let bot = Bot::from_env();
    let chat_id: i64 = std::env::var("TELEGRAM_CHAT_ID")?.parse()?;

    // let resale_matches: Vec<_> = matches.iter().filter(|m| m.is_resale).collect();

    let mut message = String::from("Reventes disponibles:\n");
    for m in matches {
        message.push_str(&format!("- {}\n", m.title));
    }

    bot.send_message(ChatId(chat_id), message).await?;
    Ok(())
}

// async fn test_sending() -> Result<(), Box<dyn std::error::Error>> {
//     dotenvy::dotenv().ok();
//     let bot = Bot::from_env();
//     let chat_id: i64 = std::env::var("TELEGRAM_CHAT_ID")?.parse()?;

//     bot.send_message(ChatId(chat_id), "Test message from Rust!")
//         .await?;
//     Ok(())
// }
// The [cfg(not(target_arch = "wasm32"))] above prevent building the tokio::main function
// for wasm32 target, because tokio isn't compatible with wasm32.
// If you aren't building for wasm32, you don't need that line.
// The two lines below avoid the "'main' function not found" error when building for wasm32 target.
#[cfg(target_arch = "wasm32")]
fn main() {}
