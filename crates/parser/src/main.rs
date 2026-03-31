use parser::interface::match_manager;
use parser::core::club::{Club, ClubType};

fn main() {
    let club = Club::new("Stade Rochelais".to_string(), ClubType::StadeRochelais, "https://billetterie.staderochelais.com/fr".to_string());
    // let club = Club::new("Union Bordeaux Bègles".to_string(), ClubType::UnionBordeauxBegles, "https://billetterie.ubbrugby.com/fr".to_string());
    let math_list = match_manager::get_seats_from_rugby_matches(Some(club));
    // let mamath_listtch_lst: Vec<parser::core::encounter::Encounter> = match_manager::get_seats_from_basketball_matches(Some(club));

    for encounter in math_list {
        println!("Match: {}", encounter.title);
        for seat in encounter.seats.unwrap_or_default() {
            println!("  Seat: {:?}, Price: {:?}", seat.seat_info, seat.price);
        }
    }
}