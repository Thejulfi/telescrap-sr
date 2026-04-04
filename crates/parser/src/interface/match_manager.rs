use crate::core::{
    club::{
        Club,
        ClubType
    },
    seat::Seat,
    encounter::{
        Encounter,
        MatchNature,
    },
};

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

/// Fetches all La Rochelle matches, optionally filtered by match nature.
///
/// # Arguments
/// * `match_type` - Optional filter to return only matches of a specific nature
pub fn get_matches_from_type_and_club(match_type: Option<MatchNature>, club: Option<Club>) -> Vec<Encounter> {
    // default club is la rochelle
    let club = club.unwrap_or_else(|| Club::new("Stade Rochelais".to_string(), crate::core::club::ClubType::StadeRochelais, "https://billetterie.staderochelais.com/fr".to_string()));
    let client = WebClient::new();
    get_matches(&club, &client, match_type)
}

/// Fetches all available seats for a list of matches.
///
/// # Arguments
/// * `matches` - The list of encounters to retrieve seats for
pub fn get_seats_from_match(matches: Vec<Encounter>) -> Vec<Seat> {
    let client = WebClient::new();
    get_seats(matches, &client)

}

/// Fetches all available seats for a single encounter.
///
/// # Arguments
/// * `encounter` - The match to retrieve seats for
pub fn get_seat_by_match(encounter: Encounter) -> Vec<Seat> {
    let client = WebClient::new();
    get_seats(vec![encounter], &client)
}

/// Fetches all available seats for all available rugby matches
///
pub fn get_seats_from_rugby_matches(club: Option<Club>) -> Vec<Encounter> {
    let client = WebClient::new();
    let matches = get_matches_from_type_and_club(Some(MatchNature::Rugby), club);
    get_encounters_with_seats(matches, &client)
}

/// Fetches all available seats for all available basketball matches
///
pub fn get_seats_from_basketball_matches(club: Option<Club>) -> Vec<Encounter> {
    let client = WebClient::new();
    let matches = get_matches_from_type_and_club(Some(MatchNature::Basketball), club);
    get_encounters_with_seats(matches, &client)
}

/// Fetches all available seats for all available matches
///
/// # Arguments
/// * `match_type` - Optional filter to return only matches of a specific nature
/// * `club` - Optional filter to return only matches for a specific club
pub fn get_seats_from_type_and_club(match_type: Option<MatchNature>, club: Option<Club>) -> Vec<Encounter> {
    let client = WebClient::new();
    let matches = get_matches_from_type_and_club(match_type, club);
    get_encounters_with_seats(matches, &client)
}

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

// fn add_to_cart(action: &SeatAction) -> Result<(), String> {
//     let client = WebClient::new();
//     client.add_to_cart(action)
// }

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

fn get_seats(matches: Vec<Encounter>, client: &impl FetchHtml) -> Vec<Seat> {
    let mut seats = Vec::new();
    for encounter in matches {
        let parser: &dyn ParseSeat = match encounter.club_type {
            ClubType::StadeRochelais => &LarochellSeatParser,
            ClubType::UnionBordeauxBegles => todo!("Bordeaux parser not yet implemented"),
        };
        if let Some(link) = &encounter.resale_link {
            match extract_html(link, client) {
                Ok(html) => seats.extend(
                    parser.parse_seat(&html, encounter)
                ),
                Err(e) => eprintln!("Error fetching {}: {}", link, e),
            }
        }
    }
    seats
}

fn get_matches(club: &Club, client: &impl FetchHtml, match_type: Option<MatchNature>) -> Vec<Encounter> {

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
    if let Some(nature) = match_type {
        matches.into_iter().filter(|encounter| encounter.nature == nature).collect()
    } else {
        matches
    }
}