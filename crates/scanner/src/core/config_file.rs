/// This module handles deserializing a `config_scan.json` file into a `ScanConfig`.
///
/// The JSON format uses plain string enums and a flat list of filter descriptors,
/// keeping the domain types (`ScanConfig`, `FilterChain`, `Club`…) free of serde concerns.
/// Conversion is done via `TryFrom<ScanConfigRaw>`.
///
/// # `config_scan.json` reference
///
/// Place this file at the root of the workspace (next to `Cargo.toml`).
///
/// ## Top-level fields
///
/// | Field          | Type              | Values                                      | Description                                      |
/// |----------------|-------------------|---------------------------------------------|--------------------------------------------------|
/// | `mode`         | string            | `"Passive"` · `"Aggressive"`                | Passive: notify only. Aggressive: auto add-to-cart. |
/// | `interval`     | integer (seconds) | e.g. `30`, `60`, `120`                      | Delay between two consecutive scans.             |
/// | `club`         | string            | `"StadeRochelais"` · `"UnionBordeauxBegles"`| Club whose ticketing site is scraped.            |
/// | `nature`       | string            | `"Rugby"` · `"Basketball"` · `"Other"`      | Match category to look for.                      |
/// | `is_preview`   | boolean           | `true` · `false`                            | Fetch seat preview images before notifying.      |
/// | `filter_chain` | array or `null`   | see filter types below                      | Ordered list of filters applied to results.      |
///
/// ## Filter types (`filter_chain` entries)
///
/// Every entry **must** have a `"type"` field that selects the filter kind.
/// All other fields are optional (`null` means "no constraint").
///
/// ### `"Price"` — keep seats within a price range
/// ```json
/// { "type": "Price", "min": 20.0, "max": 80.0 }
/// ```
/// | Field | Type           | Description                          |
/// |-------|----------------|--------------------------------------|
/// | `min` | float or `null`| Minimum seat price (inclusive).      |
/// | `max` | float or `null`| Maximum seat price (inclusive).      |
///
/// ### `"Encounter"` — keep only encounters whose title contains a substring
/// ```json
/// { "type": "Encounter", "name": "STADE ROCHELAIS" }
/// ```
/// | Field  | Type            | Description                                         |
/// |--------|-----------------|-----------------------------------------------------|
/// | `name` | string or `null`| Case-sensitive substring matched against the title. |
///
/// ### `"Seat"` — filter by seat location and/or minimum consecutive count
/// ```json
/// { "type": "Seat", "category": "Tribune", "bloc": "B", "row": "12", "min_consecutive": 2 }
/// ```
/// | Field             | Type              | Description                                                  |
/// |-------------------|-------------------|--------------------------------------------------------------|
/// | `category`        | string or `null`  | Seat category (partial, case-insensitive match).             |
/// | `bloc`            | string or `null`  | Bloc identifier (partial, case-insensitive match).           |
/// | `row`             | string or `null`  | Row label (exact, case-insensitive match).                   |
/// | `min_consecutive` | integer or `null` | Minimum number of adjacent seats required in the same row.   |
///
/// ## Full example
///
/// ```json
/// {
///   "mode": "Passive",
///   "interval": 45,
///   "club": "StadeRochelais",
///   "nature": "Rugby",
///   "is_preview": true,
///   "filter_chain": [
///     { "type": "Encounter", "name": "STADE ROCHELAIS" },
///     { "type": "Price",     "min": 20.0, "max": 80.0 },
///     { "type": "Seat",      "category": "Tribune", "bloc": null, "row": null, "min_consecutive": 2 }
///   ]
/// }
/// ```
use std::sync::Arc;

use filter::filter::{
    config::{encounter::EncounterFilter, price::PriceFilter, seat::SeatPositionFilter},
    filter_chain::FilterChain,
};
use parser::core::{
    club::{Club, ClubType},
    encounter::MatchNature,
    seat::SeatComposition,
};
use serde::Deserialize;

use crate::core::scan::{ScanConfig, ScanMode};

// ---------------------------------------------------------------------------
// Raw DTOs — only used for JSON deserialization
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub enum ScanModeRaw {
    Passive,
    Aggressive,
}

#[derive(Debug, Deserialize)]
pub enum ClubRaw {
    StadeRochelais,
    UnionBordeauxBegles,
}

#[derive(Debug, Deserialize)]
pub enum NatureRaw {
    Rugby,
    Basketball,
    Other,
}

/// One filter entry in the JSON `filter_chain` array.
/// The `"type"` field acts as the discriminant tag.
///
/// Example entries:
/// ```json
/// { "type": "Price",     "min": 10.0, "max": 80.0 }
/// { "type": "Encounter", "name": "STADE ROCHELAIS" }
/// { "type": "Seat",      "category": "Tribune", "min_consecutive": 2 }
/// ```
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum FilterDescriptor {
    Price {
        min: Option<f64>,
        max: Option<f64>,
    },
    Encounter {
        name: Option<String>,
    },
    Seat {
        category: Option<String>,
        bloc: Option<String>,
        row: Option<String>,
        min_consecutive: Option<usize>,
    },
}

/// Full raw representation of `config_scan.json`.
///
/// Example file:
/// ```json
/// {
///   "mode": "Passive",
///   "interval": 50,
///   "club": "StadeRochelais",
///   "nature": "Rugby",
///   "is_preview": true,
///   "filter_chain": [
///     { "type": "Encounter", "name": "STADE ROCHELAIS" },
///     { "type": "Price", "min": 10.0, "max": 80.0 },
///     { "type": "Seat", "category": "Tribune", "min_consecutive": 2 }
///   ]
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct ScanConfigRaw {
    pub mode: ScanModeRaw,
    pub interval: u64,
    pub club: ClubRaw,
    pub nature: NatureRaw,
    pub is_preview: bool,
    pub filter_chain: Option<Vec<FilterDescriptor>>,
}

// ---------------------------------------------------------------------------
// Conversion into domain types
// ---------------------------------------------------------------------------

impl TryFrom<ScanConfigRaw> for ScanConfig {
    type Error = String;

    fn try_from(raw: ScanConfigRaw) -> Result<Self, Self::Error> {
        let mode = match raw.mode {
            ScanModeRaw::Passive => ScanMode::PassiveScan,
            ScanModeRaw::Aggressive => ScanMode::AggressiveScan,
        };

        let club = match raw.club {
            ClubRaw::StadeRochelais => Club::new(
                "Stade Rochelais".to_string(),
                ClubType::StadeRochelais,
                "https://billetterie.staderochelais.com/fr".to_string(),
            ),
            ClubRaw::UnionBordeauxBegles => Club::new(
                "Union Bordeaux Bègles".to_string(),
                ClubType::UnionBordeauxBegles,
                "https://billetterie.ubbrugby.com/fr".to_string(),
            ),
        };

        let nature = match raw.nature {
            NatureRaw::Rugby => MatchNature::Rugby,
            NatureRaw::Basketball => MatchNature::Basketball,
            NatureRaw::Other => MatchNature::Other,
        };

        let filter_chain = raw.filter_chain.map(|descriptors| {
            let chain = descriptors
                .into_iter()
                .fold(FilterChain::new(), |chain, desc| match desc {
                    FilterDescriptor::Price { min, max } => chain.add(PriceFilter::new(min, max)),
                    FilterDescriptor::Encounter { name } => chain.add(EncounterFilter::new(name)),
                    FilterDescriptor::Seat {
                        category,
                        bloc,
                        row,
                        min_consecutive,
                    } => {
                        let composition = if category.is_some() || bloc.is_some() || row.is_some() {
                            Some(SeatComposition {
                                category: category.unwrap_or_default(),
                                bloc: bloc.unwrap_or_default(),
                                row: row.unwrap_or_default(),
                                seat_number: 0,
                            })
                        } else {
                            None
                        };
                        chain.add(SeatPositionFilter::new(composition, min_consecutive))
                    }
                });
            Arc::new(chain)
        });

        Ok(ScanConfig {
            mode,
            interval: raw.interval,
            club,
            nature,
            is_preview: raw.is_preview,
            filter_chain,
        })
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Reads and parses `path` as a `config_scan.json`, returning a ready-to-use `ScanConfig`.
/// Errors are human-readable strings describing what went wrong (I/O or JSON parse).
pub fn load_from_file(path: &str) -> Result<ScanConfig, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config file '{}': {}", path, e))?;
    let raw: ScanConfigRaw = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config file '{}': {}", path, e))?;
    ScanConfig::try_from(raw)
}
