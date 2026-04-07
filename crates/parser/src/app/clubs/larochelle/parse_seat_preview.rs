use curl::easy::Easy;
use crate::app::clubs::parsers::ParseSeatPreview;
use crate::core::seat::SeatComposition;

const BLOCK_CODES: &[(&str, &str)] = &[
    ("CHARENTE MARITIME CENTRALE BASSE",    "SUD - CHARENTE MAR CE BASSE"),
    ("MACIF L",    "NORD - MACIF EL"),
    ("MACIF O",    "NORD - MACIF CO"),
];

/// Private module containing the JSON structures used for deserializing the Pacifa3d API responses.
mod pacifa3d {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Config {
        pub accesses: Vec<AccessRef>,
    }

    #[derive(Deserialize)]
    pub struct AccessRef {
        pub name: String,
        #[serde(rename = "configURL")]
        pub config_url: String,
    }

    #[derive(Deserialize)]
    pub struct Block {
        pub children: Vec<AccessNode>,
    }

    #[derive(Deserialize)]
    pub struct AccessNode {
        pub children: Vec<RowNode>,
    }

    #[derive(Deserialize)]
    pub struct RowNode {
        #[serde(rename = "_id")]
        pub id: String,
        pub children: Vec<SeatRef>,
    }

    #[derive(Deserialize)]
    pub struct SeatRef {
        pub name: String,
        #[serde(rename = "configURL")]
        pub config_url: String,
    }

    #[derive(Deserialize)]
    pub struct SeatData {
        pub src: Vec<SeatSrc>,
    }

    #[derive(Deserialize)]
    pub struct SeatSrc {
        #[serde(rename = "type")]
        pub src_type: String,
        pub url: String,
    }
}

const PACIFA3D_BASE: &str = "https://static.pacifa3d.com/StadeMarcelDeflandreLAROCHELLE/WIMS/STADE-ROCHELAIS_RUGBY_v2025-2026/";

/// Resolves a relative Pacifa3d URL (starting with `./`) against the base URL.
fn pacifa3d_resolve_url(relative: &str) -> String {
    format!("{}{}", PACIFA3D_BASE, relative.trim_start_matches("./"))
}

/// Performs a simple HTTP GET and returns the response body as a String.
fn pacifa3d_fetch_json(url: &str) -> Option<String> {
    let mut easy = Easy::new();
    easy.url(url).ok()?;
    easy.follow_location(true).ok()?;
    let mut body = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            body.extend_from_slice(data);
            Ok(data.len())
        }).ok()?;
        transfer.perform().ok()?;
    }
    String::from_utf8(body).ok()
}

/// Resolves the Pacifa3d preview image URL for a given seat composition.
///
/// Performs three sequential HTTP GET calls against the Pacifa3d static API:
/// 1. Main config → locate the matching block by access name
/// 2. Block JSON  → locate the row and seat by row id and seat number
/// 3. Seat JSON   → extract the panorama ID and build the preview URL
///
/// Returns `None` if any step fails (network error, missing data, no match).
pub fn fetch_preview_url(composition: &SeatComposition) -> Option<String> {
    // Translate the scraper name to the exact Pacifa3d access name.
    // Look up by category first (more specific), then by access, then use access as-is.
    let pacifa3d_access = BLOCK_CODES
        .iter()
        .find(|(scraper_name, _)| scraper_name.eq_ignore_ascii_case(&composition.category))
        .or_else(|| BLOCK_CODES.iter().find(|(scraper_name, _)| scraper_name.eq_ignore_ascii_case(&composition.bloc)))
        .map(|(_, pacifa3d_name)| *pacifa3d_name)
        .unwrap_or(composition.bloc.as_str());

    // Step 1: fetch the main config and find the block matching the access name
    let config_url = format!("{}d/STADE-ROCHELAIS_RUGBY_v2025-2026.json", PACIFA3D_BASE);
    let config: pacifa3d::Config = serde_json::from_str(&pacifa3d_fetch_json(&config_url)?).ok()?;

    let access_ref = config.accesses.iter().find(|a| a.name == pacifa3d_access)?;

    // Step 2: fetch the block JSON and navigate to the matching seat
    let block_url = pacifa3d_resolve_url(&access_ref.config_url);
    let block: pacifa3d::Block = serde_json::from_str(&pacifa3d_fetch_json(&block_url)?).ok()?;

    let rows = &block.children.first()?.children;
    let row = rows.iter().find(|r| r.id == composition.row)?;
    let seat_name = composition.seat_number.to_string();
    let seat_ref = row.children.iter().find(|s| s.name == seat_name)?;

    // Step 3: fetch the seat JSON and extract the panorama ID
    let seat_url = pacifa3d_resolve_url(&seat_ref.config_url);
    let seat: pacifa3d::SeatData = serde_json::from_str(&pacifa3d_fetch_json(&seat_url)?).ok()?;

    let pano_url = seat.src.iter().find(|s| s.src_type == "PANO")?.url.as_str();
    let pano_id = pano_url.split('/').last()?.trim_end_matches(".xml");

    Some(format!("{}p/panos/{}.tiles/pano_f.jpg", PACIFA3D_BASE, pano_id))
}

/// Larochelle implementation of the `ParseSeatPreview` trait.
pub struct LarochellSeatPreviewParser;

impl ParseSeatPreview for LarochellSeatPreviewParser {
    fn fetch_preview_url(&self, composition: &SeatComposition) -> Option<String> {
        fetch_preview_url(composition)
    }
}
