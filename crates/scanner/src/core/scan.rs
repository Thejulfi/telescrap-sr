/// This module defines the core structures and logic for scanning encounters based on specified configurations and filters.
use std::time::SystemTime;
use parser::core::{
    club::Club,
    encounter::{Encounter, MatchNature},
    seat::SeatComposition,
};

/// Represents the mode of scanning, which can be either passive or aggressive.
#[derive(Debug, Clone, PartialEq)]
pub enum ScanMode {
    PassiveScan,
    AggressiveScan,
}

/// Represents the filter criteria for scanning encounters, including price thresholds, date ranges, seat positions, and title matching.
#[derive(Debug, Clone)]
pub struct ScanFilter {
    pub price_threshold: Option<f64>,
    pub date_range: Option<(String, String)>,
    pub position: Option<SeatComposition>,
    pub side_by_side: Option<u64>,
    pub match_title: Option<String>,
    pub is_preview: Option<bool>,
}

/// Represents the configuration for scanning encounters, including the mode, interval, club, match nature, and optional filters.
#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub mode: ScanMode,
    pub interval: u64,
    pub club: Club,
    pub nature: MatchNature,
    pub filter: Option<ScanFilter>,
}

/// Represents the result of a scan, containing the list of encounters found and the timestamp of when the scan was performed.
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub encounters: Vec<Encounter>,
    pub scanned_at: SystemTime,
}

impl ScanConfig {
    /// Creates a new `ScanConfig` with the specified parameters.
    /// 
    /// # Arguments
    /// * `mode` - The mode of scanning (passive or aggressive).
    /// * `interval` - The interval in seconds between scans.
    /// * `club` - The club for which the scan is being performed.
    /// * `nature` - The nature of the match (e.g., rugby, basketball).
    /// * `filter` - Optional filter criteria for the scan.
    /// 
    /// # Returns
    /// A new instance of `ScanConfig` initialized with the provided parameters.
    pub fn new(mode: ScanMode, interval: u64, club: Club, nature: MatchNature, filter: Option<ScanFilter>) -> Self {
        Self { mode, interval, club, nature, filter }
    }
}

impl ScanResult {
    /// Creates a new `ScanResult` with the specified list of encounters.
    ///
    /// # Arguments
    /// * `encounters` - A vector of `Encounter` instances representing the results of the scan.
    /// 
    /// # Returns
    /// A new instance of `ScanResult` initialized with the provided encounters and the current timestamp.
    pub fn new(encounters: Vec<Encounter>) -> Self {
        Self { encounters, scanned_at: SystemTime::now() }
    }
}