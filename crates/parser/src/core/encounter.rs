use crate::core::seat::Seat;
use crate::core::club::ClubType;

#[derive(Debug, PartialEq, Clone)]
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
    pub club_type: ClubType,
    pub title: String,
    pub date: String,
    pub nature: MatchNature,
    pub resale_link: Option<String>,
    pub seats: Option<Vec<Seat>>,
}

impl Encounter {
    pub fn set_seats(&mut self, seats: Vec<Seat>) {
        self.seats = Some(seats);
    }
}