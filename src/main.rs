#![allow(unused)]
use parser::{
    controller::encounter_store::StoreEncounters,
    core::{
        club::ClubType,
        encounter::{Encounter, MatchNature},
    },
    interface::storage::EncounterStore,
};
use scanner::{controller::notify::Notify, core::scan};
use scanner::interface::runner::ScannerHandle;
use telegram_notifier::TelegramNotifier;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    // Load Telegram bot token and chat ID from environment variables
    let bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
        .expect("TELEGRAM_BOT_TOKEN non défini dans .env");
    let chat_id: i64 = std::env::var("TELEGRAM_CHAT_ID")
        .expect("TELEGRAM_CHAT_ID non défini dans .env")
        .parse()
        .expect("TELEGRAM_CHAT_ID doit être un entier");

    // Seed the database with the LR / BDX match to track, then drop the handle
    // so the scanner can open its own connection without hitting DatabaseAlreadyOpen.
    {
        let db = EncounterStore::open("matchs.db").unwrap();
        db.upsert(&Encounter::new(
            ClubType::StadeRochelais,
            "STADE ROCHELAIS / UNION BORDEAUX BÈGLES".to_string(),
            "samedi 18 avril à 14h30".to_string(),
            MatchNature::Rugby,
            Some("https://billetterie.staderochelais.com/fr/product/1048/revente_stade_rochelais_union_bordeaux_begles".to_string()),
        )).unwrap();
    }
    // {
    //     let db = EncounterStore::open("matchs.db").unwrap();
    //     db.upsert(&Encounter::new(
    //         ClubType::StadeRochelais,
    //         "STADE ROCHELAIS BASKET / ROUEN".to_string(),
    //         "mardi 7 avril à 20h00".to_string(),
    //         MatchNature::Basketball,
    //         Some("https://billetterie.staderochelais.com/fr/product/1032/revente_stade_rochelais_basket_rouen".to_string()),
    //     )).unwrap();
    // }

    // Telegram notifier configuration with environment variables
    let notifier = TelegramNotifier::new(bot_token, chat_id, env!("CARGO_PKG_VERSION"));
    // Send or update the bot's status message on startup
    // Scanner configuration (interval, club, match type, filters)
    let scan_config = ScannerHandle::configure();
    notifier.notify_state(scan_config.clone());
    // Start the scanner with the specified configuration and notifier
    let notifier_for_shutdown = notifier.clone();
    let _handle = ScannerHandle::start(scan_config, notifier);

    tokio::signal::ctrl_c().await.unwrap();
}
