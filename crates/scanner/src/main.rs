
use scanner::core::scan::ScanConfig;
use scanner::interface::runner::ScannerHandle;
use scanner::interface::notifiers::console::ConsoleNotifier;

#[tokio::main]
async fn main() {
    let scan_config = ScanConfig::new(60, parser::core::club::Club::new(
        "Stade Rochelais".to_string(),
        parser::core::club::ClubType::StadeRochelais,
        "https://billetterie.staderochelais.com/fr".to_string(),
    ));

    let _scanner_handle = ScannerHandle::start(scan_config, ConsoleNotifier);

    // Garde le process en vie jusqu'à Ctrl+C
    tokio::signal::ctrl_c().await.unwrap();
}
