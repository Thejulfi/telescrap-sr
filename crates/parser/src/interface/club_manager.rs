use crate::core::club::{Club, ClubType};

pub fn new(club_type: ClubType) -> Club {
    match club_type {
        ClubType::StadeRochelais => Club::new("Stade Rochelais".to_string(), ClubType::StadeRochelais, "https://billetterie.staderochelais.com/fr".to_string()),
        ClubType::UnionBordeauxBegles => Club::new("Union Bordeaux Bègles".to_string(), ClubType::UnionBordeauxBegles, "https://billetterie.ubbrugby.com/fr".to_string()),
    }
}
