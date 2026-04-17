/// This module defines the core structures and logic for scanning encounters based on specified configurations and filters.
use std::sync::Arc;
use std::time::SystemTime;
use filter::filter::filter_chain::FilterChain;
use filter::filter::config::price::PriceFilter;
use filter::filter::config::encounter::EncounterFilter;
use parser::core::{
    club::{Club, ClubType},
    encounter::{Encounter, MatchNature},
};

/// Represents the mode of scanning, which can be either passive or aggressive.
#[derive(Debug, Clone, PartialEq)]
pub enum ScanMode {
    PassiveScan,
    AggressiveScan,
}

/// Represents the configuration for scanning encounters, including the mode, interval, club, match nature, and optional filters.
#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub mode: ScanMode,
    pub interval: u64,
    pub club: Club,
    pub nature: MatchNature,
    // pub match_title: Option<String>,
    pub is_preview: bool,
    pub filter_chain: Option<Arc<FilterChain>>,
}

/// Represents the result of a scan, containing the list of encounters found and the timestamp of when the scan was performed.
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub encounters: Vec<Encounter>,
    pub scanned_at: SystemTime,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            mode: ScanMode::PassiveScan,
            interval: 120,
            club: Club::new(
                "Stade Rochelais".to_string(),
                ClubType::StadeRochelais,
                "https://billetterie.staderochelais.com/fr".to_string(),
            ),
            nature: MatchNature::Basketball,
            // match_title: Some("STADE ROCHELAIS / UNION BORDEAUX BÈGLES".to_string()),
            is_preview: false,
            filter_chain: Some(Arc::new(
                FilterChain::new()
                .add(EncounterFilter::new(Some("STADE ROCHELAIS BASKET / SAINT-CHAMOND".to_string())))
                .add(PriceFilter::new(Some(10.0), None))
            )),
        }
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