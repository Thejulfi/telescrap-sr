use parser::core::club::Club;
use parser::core::encounter::{Encounter, MatchNature};
use std::time::SystemTime;
use parser::core::seat::SeatComposition;

#[derive(Debug, Clone, PartialEq)]
pub enum ScanMode {
    PassiveScan,
    AggressiveScan,
}

#[derive(Debug, Clone)]
pub struct ScanFilter {
    pub price_threshold: Option<f64>,
    pub date_range: Option<(String, String)>,
    pub position: Option<SeatComposition>,
    pub side_by_side: Option<u64>,
    pub match_title: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub mode: ScanMode,
    pub interval: u64,
    pub club: Club,
    pub nature: MatchNature,
    pub filter: Option<ScanFilter>,
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub encounters: Vec<Encounter>,
    pub scanned_at: SystemTime,
}

impl ScanConfig {
    pub fn new(mode: ScanMode, interval: u64, club: Club, nature: MatchNature, filter: Option<ScanFilter>) -> Self {
        Self { mode, interval, club, nature, filter }
    }
}

impl ScanResult {
    pub fn new(encounters: Vec<Encounter>) -> Self {
        Self { encounters, scanned_at: SystemTime::now() }
    }
}