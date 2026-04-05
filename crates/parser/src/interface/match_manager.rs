use crate::{controller::encounter_store::StoreEncounters, core::{
    club::{
        Club,
        ClubType
    }, encounter::{
        Encounter,
        MatchNature,
    }, seat::Seat
}};

use crate::controller::html_extract::{FetchHtml, extract_html};

use crate::app::clubs::{
    parsers::{
        ParseSeat,
        ParseMatch,
    },
    larochelle::{
        parse_match::LarochellMatchParser,
        parse_seat::LarochellSeatParser,
    }
};

use crate::interface::curl::web::{WebClient, connect_and_add_to_cart};
use crate::interface::storage::redb::EncounterStore;

/// Connects to the shop with the given seat information and adds it to the cart.
/// 
/// # Arguments
/// * `seat` - The seat to add to the cart
/// # Returns
/// Ok(()) if the seat was successfully added to the cart, Err(String) with an
/// error message if there was an issue during the process
pub fn connect_and_add_seat_to_cart(email: String, password: String, seat: Seat) -> Result<(), Box<dyn std::error::Error>> {
    connect_and_add_to_cart(&email, &password, &seat.actions)
}

/// Prints all encounter records stored in the database.
pub fn print_db_contents() {
    let db = match EncounterStore::open("matchs.db") {
        Ok(db) => db,
        Err(e) => { eprintln!("Failed to open database: {}", e); return; }
    };
    match db.get_all() {
        Ok(records) if records.is_empty() => println!("Database is empty."),
        Ok(records) => {
            println!("Database contains {} record(s):", records.len());
            for r in records {
                println!(
                    "  [{active}] {title} | {date} | club: {club} | link: {link}",
                    active = if r.resale_active { "active" } else { "inactive" },
                    title = r.title,
                    date = r.date,
                    club = r.club_type,
                    link = r.resale_link,
                );
            }
        }
        Err(e) => eprintln!("Storage error: {}", e),
    }
}

/// Fetches encounters with seats for a given club and match nature
/// 
/// # Arguments
/// * `club` - The club to fetch matches for
/// * `match_type` - The nature of the match to fetch
/// # Returns
/// A list of encounters with their seats information populated
/// 
pub fn get_seats_from_matches(club: Club, match_type: MatchNature) -> Vec<Encounter> {
    let client = WebClient::new();
    let db = EncounterStore::open("matchs.db").unwrap();
    let matches = get_matches_from_type_and_club(match_type, club);
    for encounter in &matches {
        if let Err(e) = db.upsert(encounter) {
            eprintln!("Storage error for '{}': {}", encounter.title, e);
        }
    }
    get_encounters_with_seats(matches, &client)
}

/// Fectes match's seats from a given match title, club and match nature.
/// It first tries to find an active resale link in the database for the given title, if it finds one it fetches seats from it,
/// otherwise it falls back to fetching matches from the web and filtering by title.
/// 
/// # Arguments
/// * `match_title` - The title of the match to fetch seats for
/// * `club` - The club to fetch matches for
/// * `match_type` - The nature of the match to fetch
/// # Returns
/// A list of encounters with their seats information populated (which is 1 if a match with the given title is found, 0 otherwise)
/// 
pub fn get_seats_from_match_title(match_title: String, club: Club, match_type: MatchNature) -> Vec<Encounter> {
    let client = WebClient::new();
    let db = EncounterStore::open("matchs.db").unwrap();

    // Get all occurence from data base
    match db.get_all() {
        // If there is something
        Ok(records) => {
            // try to find a record with the same title and an active resale link
            for record in records {
                if record.title == match_title && record.resale_active {
                        // If that records has an active resale link, try to fetch seats from it
                        let link = &record.resale_link;
                        match extract_html(link, &client) {
                            Ok(_) => {
                                let enc = Encounter::new(
                                    Club::get_type_from_name(&record.club_type),
                                    record.title,
                                    record.date,
                                    match_type,
                                    Some(record.resale_link));

                                // Stop here, we return the seats from this match (vector of 1 encounter)
                                return get_encounters_with_seats(vec![enc], &client);
                            }
                            Err(e) => eprintln!("Error fetching {}: {}", link, e),
                    }
                }
            }
        }
        Err(e) => eprintln!("Storage error while retrieving matches: {}", e),
    }

    // If there is no resale link, get matches
    let matches = get_matches_from_type_and_club(match_type, club);
    // Filter by the one with the right title
    let filtered = matches.into_iter().filter(|e| e.title == match_title).collect();
    // Get seats from the filtered match (vector of 0 or 1 encounter)
    get_encounters_with_seats(filtered, &client)
}

/// Internal function to fetch seats for a list of encounters, given a client to fetch HTML content.
/// 
/// # Arguments
/// * `matches` - The list of encounters to retrieve seats for
/// * `client` - The client to use for fetching HTML content
/// # Returns
/// A list of encounters with their seats information populated
fn get_encounters_with_seats(matches: Vec<Encounter>, client: &impl FetchHtml) -> Vec<Encounter> {
    matches.into_iter().map(|mut encounter| {
        let parser: &dyn ParseSeat = match encounter.club_type {
            ClubType::StadeRochelais => &LarochellSeatParser,
            ClubType::UnionBordeauxBegles => todo!("Bordeaux parser not yet implemented"),
        };
        if let Some(link) = encounter.resale_link.clone() {
            match extract_html(&link, client) {
                Ok(html) => encounter.set_seats(parser.parse_seat(&html, encounter.clone())),
                Err(e) => eprintln!("Error fetching {}: {}", link, e),
            }
        }
        encounter
    }).collect()
}

/// Internal function to fetch matches for a given club and client, optionally filtered by match nature.
/// 
/// # Arguments
/// * `club` - The club to fetch matches for
/// * `client` - The client to use for fetching HTML content
/// * `match_type` - Optional filter to return only matches of a specific nature
/// # Returns
/// A list of encounters matching the specified criteria
/// 
fn get_matches(club: &Club, client: &impl FetchHtml, match_type: MatchNature) -> Vec<Encounter> {

    // Step 0: Set the correct parser from the club
    let parser: &dyn ParseMatch = match club.club_type {
        ClubType::StadeRochelais => &LarochellMatchParser,
        ClubType::UnionBordeauxBegles => todo!("Bordeaux parser not yet implemented"),
    };

    // Step 1 : Extract HTML content from the club's URL
    let content = extract_html(club.get_url(), client);

    // Step 2 : Parse the HTML content to find matches (encounters) information
    let matches = parser.parse_match(&content.unwrap_or_default());

    // Step 3 : filter by nature you want to get
    matches.into_iter().filter(|encounter| encounter.nature == match_type).collect()
}

/// Internal function to fetch all match encounters for a given club and match nature
///
/// # Arguments
/// * `match_type` - Optional filter to return only matches of a specific nature
/// * `club` - The club to fetch matches for
/// # Returns
/// A list of encounters matching the specified criteria
fn get_matches_from_type_and_club(match_type: MatchNature, club: Club) -> Vec<Encounter> {
    let client = WebClient::new();
    get_matches(&club, &client, match_type)
}