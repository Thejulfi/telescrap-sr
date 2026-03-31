use scanner::core::scan::ScanConfig;
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

    let scan_config = ScanConfig::new(
        60,
        parser::core::club::Club::new(
            "Stade Rochelais".to_string(),
            parser::core::club::ClubType::StadeRochelais,
            "https://billetterie.staderochelais.com/fr".to_string(),
        ),
    );

    let notifier = TelegramNotifier::new(bot_token, chat_id);
    let _handle = ScannerHandle::start(scan_config, notifier);

    tokio::signal::ctrl_c().await.unwrap();
}
