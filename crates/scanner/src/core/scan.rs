use parser::core::club::Club;
use parser::core::encounter::Encounter;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub interval: u64,
    pub club: Club,
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub encounters: Vec<Encounter>,
    pub scanned_at: SystemTime,
}

impl ScanConfig {
    pub fn new(interval: u64, club: Club) -> Self {
        Self { interval, club }
    }
}

impl ScanResult {
    pub fn new(encounters: Vec<Encounter>) -> Self {
        Self { encounters, scanned_at: SystemTime::now() }
    }
}