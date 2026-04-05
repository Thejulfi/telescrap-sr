use crate::core::seat::Seat;
use crate::core::club::ClubType;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MatchNature {
    Rugby,
    Basketball,
    Other,
}

impl MatchNature {
    pub fn from_title(title: &str) -> Self {
        let lower = title.to_lowercase();
        if lower.contains("basket") {
            MatchNature::Basketball
        } else if lower.starts_with("stade rochelais") {
            MatchNature::Rugby
        } else {
            MatchNature::Other
        }
    }
}

#[derive(Debug, Clone)]
pub struct Encounter {
    pub id: u64,
    pub club_type: ClubType,
    pub title: String,
    pub date: String,
    pub nature: MatchNature,
    pub resale_link: Option<String>,
    pub seats: Option<Vec<Seat>>,
}

static ENCOUNTER_COUNTER: AtomicU64 = AtomicU64::new(1);

impl Encounter {
    pub fn new(club_type: ClubType, title: String, date: String, nature: MatchNature, resale_link: Option<String>) -> Self {
        Self {
            id: ENCOUNTER_COUNTER.fetch_add(1, Ordering::Relaxed),
            club_type,
            title,
            date,
            nature,
            resale_link,
            seats: None,
        }
    }

    pub fn set_seats(&mut self, seats: Vec<Seat>) {
        self.seats = Some(seats);
    }
}