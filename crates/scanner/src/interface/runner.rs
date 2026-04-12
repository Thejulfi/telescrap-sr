use crate::core::scan::{ScanConfig, ScanFilter, ScanMode};
use crate::controller::notify::Notify;
use crate::app::scan_task::ScanTask;
use parser::core::encounter::MatchNature;
#[allow(unused)]
use parser::core::seat::SeatComposition;
use tokio::sync::watch;

pub struct ScannerHandle {
    abort_handle: tokio::task::AbortHandle,
}

impl ScannerHandle {
    pub fn configure() -> ScanConfig {
        ScanConfig::new(
            ScanMode::PassiveScan,
            30,
            parser::core::club::Club::new(
                "Stade Rochelais".to_string(),
                parser::core::club::ClubType::StadeRochelais,
                "https://billetterie.staderochelais.com/fr".to_string(),
            ),
            MatchNature::Rugby,
            Some(ScanFilter {
                price_threshold: None,
                date_range: None,
                position: None,
                side_by_side: None,
                match_title: None,
                is_preview: Some(true),
            }),
        )
    }
    
    pub fn start(config_rx: watch::Receiver<ScanConfig>, notifier: impl Notify) -> Self {
        let task = ScanTask::new(config_rx, notifier);
        let handle = tokio::spawn(async move { task.run().await });
        Self { abort_handle: handle.abort_handle() }
    }

    pub fn stop(&self) {
        self.abort_handle.abort();
    }
}