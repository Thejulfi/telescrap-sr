#[allow(unused_imports)]
use parser::interface::storage::redb::EncounterStore;
#[allow(unused_imports)]
use parser::controller::encounter_store::StoreEncounters;
#[allow(unused_imports)]
use parser::interface::match_manager::get_seats_from_matches;

fn main() {
    // parse_rugby_matches();
    // parse_basketball_matches();
    get_all_db_records();
}

#[allow(unused)]
fn get_all_db_records() {
    let store = EncounterStore::open("matchs.db").unwrap();
    match store.get_all() {
        Ok(records) => {
            println!("{} record(s) in DB:", records.len());
            for r in records {
                println!("\n\n  [{active}] {title} | {date} | {link}",
                    active = if r.resale_active { "active" } else { "inactive" },
                    title = r.title,
                    date = r.date,
                    link = r.resale_link,
                );
            }
        }
        Err(e) => eprintln!("DB error: {}", e),
    }
}

#[allow(unused)]
fn parse_rugby_matches() {
    let club = parser::core::club::Club::new(
        "Stade Rochelais".to_string(),
        parser::core::club::ClubType::StadeRochelais,
        "https://billetterie.staderochelais.com/fr".to_string(),
    );

    let match_list = get_seats_from_matches(
         Some(club),
        Some(parser::core::encounter::MatchNature::Rugby)
    );

    dbg!(&match_list);
}

#[allow(unused)]
fn parse_basketball_matches() {
    let club = parser::core::club::Club::new(
        "Stade Rochelais".to_string(),
        parser::core::club::ClubType::StadeRochelais,
        "https://billetterie.staderochelais.com/fr".to_string(),
    );

    let match_list = get_seats_from_matches(
         Some(club),
        Some(parser::core::encounter::MatchNature::Basketball)
    );

    dbg!(&match_list);
}

#[allow(dead_code)]
fn test_conection() {

    //     dotenvy::dotenv().ok();
    // let email = std::env::var("SHOP_EMAIL").expect("SHOP_EMAIL non défini");
    // let password = std::env::var("SHOP_PASSWORD").expect("SHOP_PASSWORD non défini");

    // // étape 1 : scan des matchs et sièges (anonyme)
    // let club = Club::new("Stade Rochelais".to_string(), ClubType::StadeRochelais, "https://billetterie.staderochelais.com/fr".to_string());
    // let match_list = match_manager::get_seats_from_basketball_matches(Some(club));

    // // étape 2 : connexion + ajout au panier du premier siège disponible
    // if let Some(encounter) = match_list.last() {
    //     println!("Match : {}", encounter.title);
    //     if let Some(seat) = encounter.seats.as_ref().and_then(|s| s.first()) {
    //         println!("Ajout au panier : pack_id={} ticket_id={}", seat.actions.pack_id, seat.actions.ticket_id);
    //         match match_manager::connect_and_add_seat_to_cart(email, password, seat.clone()) {
    //             Ok(_) => println!("✅ Billet ajouté au panier"),
    //             Err(e) => eprintln!("❌ Erreur : {}", e),
    //         }
    //     } else {
    //         println!("Aucun siège disponible pour ce match");
    //     }
    // } else {
    //     println!("Aucun match disponible");
    // }
}