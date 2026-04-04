use scanner::interface::runner::ScannerHandle;
use telegram_notifier::TelegramNotifier;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
        .expect("TELEGRAM_BOT_TOKEN non défini dans .env");
    let chat_id: i64 = std::env::var("TELEGRAM_CHAT_ID")
        .expect("TELEGRAM_CHAT_ID non défini dans .env")
        .parse()
        .expect("TELEGRAM_CHAT_ID doit être un entier");

    // Telegram notifier configuration with environment variables
    let notifier = TelegramNotifier::new(bot_token, chat_id);
    // Scanner configuration (interval, club, match type, filters)
    let scan_config = ScannerHandle::configure();
    // Start the scanner with the specified configuration and notifier
    let _handle = ScannerHandle::start(scan_config, notifier);

    tokio::signal::ctrl_c().await.unwrap();
}
