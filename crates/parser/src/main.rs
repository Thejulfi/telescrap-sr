use parser::interface::match_manager;
use parser::core::club::{Club, ClubType};

fn main() {
    dotenvy::dotenv().ok();
    let email = std::env::var("SHOP_EMAIL").expect("SHOP_EMAIL non défini");
    let password = std::env::var("SHOP_PASSWORD").expect("SHOP_PASSWORD non défini");

    // étape 1 : scan des matchs et sièges (anonyme)
    let club = Club::new("Stade Rochelais".to_string(), ClubType::StadeRochelais, "https://billetterie.staderochelais.com/fr".to_string());
    let match_list = match_manager::get_seats_from_basketball_matches(Some(club));

    // étape 2 : connexion + ajout au panier du premier siège disponible
    if let Some(encounter) = match_list.last() {
        println!("Match : {}", encounter.title);
        if let Some(seat) = encounter.seats.as_ref().and_then(|s| s.first()) {
            println!("Ajout au panier : pack_id={} ticket_id={}", seat.actions.pack_id, seat.actions.ticket_id);
            match match_manager::connect_and_add_seat_to_cart(email, password, seat.clone()) {
                Ok(_) => println!("✅ Billet ajouté au panier"),
                Err(e) => eprintln!("❌ Erreur : {}", e),
            }
        } else {
            println!("Aucun siège disponible pour ce match");
        }
    } else {
        println!("Aucun match disponible");
    }
}