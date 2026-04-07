use crate::core::scan::{ScanConfig, ScanFilter, ScanMode};
use crate::controller::notify::Notify;
use crate::app::scan_task::ScanTask;
use parser::core::encounter::MatchNature;
use parser::core::seat::SeatComposition;

pub struct ScannerHandle {
    abort_handle: tokio::task::AbortHandle,
}

impl ScannerHandle {
    pub fn configure() -> ScanConfig {
        ScanConfig::new(
            ScanMode::PassiveScan,
            60,
            parser::core::club::Club::new(
                "Stade Rochelais".to_string(),
                parser::core::club::ClubType::StadeRochelais,
                "https://billetterie.staderochelais.com/fr".to_string(),
            ),
            MatchNature::Basketball,
            Some(ScanFilter {
                price_threshold: None,
                date_range: None,
                position: Some(SeatComposition {
                    category: "".to_string(),
                    access: "".to_string(),
                    row: "".to_string(),
                    seat_number: 0,
                }),
                side_by_side: None,
                // match_title: Some("STADE ROCHELAIS BASKET / ROUEN".to_string()),
                match_title: Some("STADE ROCHELAIS / UNION BORDEAUX BÈGLES".to_string()),
            }),
        )
    }
    
    pub fn start(config: ScanConfig, notifier: impl Notify) -> Self {
        let task = ScanTask::new(config, notifier);
        let handle = tokio::spawn(async move { task.run().await });
        Self { abort_handle: handle.abort_handle() }
    }

    pub fn stop(&self) {
        self.abort_handle.abort();
    }
}