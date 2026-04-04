use parser::core::club::Club;
use parser::core::encounter::Encounter;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub enum ScanMode {
    PassiveScan,
    AggressiveScan,
}

#[derive(Debug, Clone)]
pub struct ScanFilter {
    pub price_threshold: Option<f64>,
    pub date_range: Option<(String, String)>,
    pub position: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub mode: ScanMode,
    pub interval: u64,
    pub club: Club,
    pub filter: Option<ScanFilter>,
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub encounters: Vec<Encounter>,
    pub scanned_at: SystemTime,
}

impl ScanConfig {
    pub fn new(mode: ScanMode, interval: u64, club: Club, filter: Option<ScanFilter>) -> Self {
        Self { mode, interval, club, filter }
    }
}

impl ScanResult {
    pub fn new(encounters: Vec<Encounter>) -> Self {
        Self { encounters, scanned_at: SystemTime::now() }
    }
}