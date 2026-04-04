
use scanner::core::scan::{ScanConfig, ScanMode, ScanFilter};
use scanner::interface::runner::ScannerHandle;
use scanner::interface::notifiers::console::ConsoleNotifier;

#[tokio::main]
async fn main() {
    let scan_config = ScanConfig::new(ScanMode::AggressiveScan, 60, parser::core::club::Club::new(
        "Stade Rochelais".to_string(),
        parser::core::club::ClubType::StadeRochelais,
        "https://billetterie.staderochelais.com/fr".to_string(),
    ), Some(ScanFilter {
        price_threshold: Some(50.0),
        date_range: None,
        position: None,
    }));

    let _scanner_handle = ScannerHandle::start(scan_config, ConsoleNotifier);

    // Garde le process en vie jusqu'à Ctrl+C
    tokio::signal::ctrl_c().await.unwrap();
}
