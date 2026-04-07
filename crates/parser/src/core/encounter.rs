/// This module defines the Encounter struct, which represents a sports match or event,
/// including its title, date, nature, and associated seats.
/// It also includes the MatchNature enum to categorize the type of match (e.g., rugby, basketball).
use crate::core::seat::Seat;
use crate::core::club::ClubType;
use std::sync::atomic::{AtomicU64, Ordering};

/// MatchNature is an enumeration that categorizes the type of match or event, such as rugby, basketball, or other.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MatchNature {
    Rugby,
    Basketball,
    Other,
}

/// Implementation of the MatchNature enum, including a method to determine the match nature from a title string.
impl MatchNature {

    /// Determines the MatchNature based on the content of the title string.
    /// 
    /// # Arguments
    /// * `title` - The title of the match or event as a string
    /// # Returns
    /// The corresponding MatchNature enum variant based on the title content
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

/// Encounter represents a sports match or event, including its title, date, nature, and associated seats.
/// 
/// It includes an ID for database storage, the club type, and an optional resale link for tickets.
/// 
/// The seats field is optional and can be set after the encounter is created,
/// allowing for a two-step parsing process where encounters are first identified and then detailed seat
/// information is added later.
/// 
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

/// A static atomic counter to generate unique IDs for encounters when they are created.
static ENCOUNTER_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Implementation of the Encounter struct, including a constructor and a method to set the seats information.
impl Encounter {
    /// Creates a new Encounter instance with the given club type, title, date, nature, and optional resale link.
    ///
    /// # Arguments
    /// * `club_type` - The type of the club associated with the encounter (from the ClubType enum)
    /// * `title` - The title of the encounter as a string
    /// * `date` - The date of the encounter as a string in natural language format (e.g., "Samedi 16 septembre 2023 à 14:00")
    /// * `nature` - The nature of the encounter (from the MatchNature enum)
    /// * `resale_link` - An optional resale link for the encounter's tickets
    /// # Returns
    /// A new instance of the Encounter struct with the provided information and a unique ID
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

    /// Sets the seats information for the encounter.
    /// 
    /// # Arguments
    /// * `seats` - A vector of Seat instances representing the seats available for the encounter
    /// # Returns
    /// This method does not return a value, but it updates the seats field of the Encounter instance with the provided seats information
    pub fn set_seats(&mut self, seats: Vec<Seat>) {
        self.seats = Some(seats);
    }
}