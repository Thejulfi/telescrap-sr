mod filter;

use filter::filter_chain::FilterChain;
use filter::rule::{Rule, RuleSet};
use filter::config::{
    seat::SeatPositionFilter,
    encounter::EncounterFilter,
    price::PriceFilter,
};
use filter::Filter;

use parser::core::{
    club::ClubType,
    encounter::{Encounter, MatchNature},
    seat::{Seat, SeatAction, SeatComposition, SeatInfo},
};
// use filter::is_sidebside::SideBySideFilter;

fn make_seat(category: &str, bloc: &str, row: &str, seat_number: u64, price: &str) -> Seat {
    Seat {
        seat_info: SeatInfo {
            full_name: format!("{} {} R{} P{}", category, bloc, row, seat_number),
            composition: SeatComposition {
                category: category.to_string(),
                bloc: bloc.to_string(),
                row: row.to_string(),
                seat_number,
            },
            preview_url: None,
        },
        price: price.to_string(),
        actions: SeatAction {
            pack_id: format!("PACK-{bloc}-{seat_number}"),
            ticket_id: format!("TKT-{bloc}-{seat_number}"),
            ajax_url: "https://ticketing.example.com/ajax/add-to-cart".to_string(),
            price_min: price.to_string(),
            price_max: price.to_string(),
            libraries: String::new(),
            form_build_id: format!("form-build-{seat_number}"),
            form_token: format!("token-{seat_number}"),
        },
    }
}

fn fake_encounters() -> Vec<Encounter> {
    let mut e1 = Encounter::new(
        ClubType::StadeRochelais,
        "Stade Rochelais vs Stade Toulousain".to_string(),
        "Samedi 19 avril 2025 à 21:05".to_string(),
        MatchNature::Rugby,
        Some("https://ticketing.staderochelais.com/resale/SR-TLS-2025".to_string()),
    );
    e1.seats = Some(vec![
        make_seat("Virage Nord", "VN-12", "C", 14, "35.00"),
        make_seat("Virage Nord", "VN-12", "C", 15, "35.00"),
        make_seat("Tribune Latérale", "TL-04", "B", 7, "58.00"),
    ]);

    let mut e2 = Encounter::new(
        ClubType::StadeRochelais,
        "Stade Rochelais vs Leinster Rugby".to_string(),
        "Samedi 26 avril 2025 à 16:00".to_string(),
        MatchNature::Rugby,
        Some("https://ticketing.staderochelais.com/resale/SR-LEI-2025".to_string()),
    );
    e2.seats = Some(vec![
        make_seat("Virage Sud", "VS-03", "A", 22, "42.00"),
    ]);

    let e3 = Encounter::new(
        ClubType::StadeRochelais,
        "Stade Rochelais vs Racing 92".to_string(),
        "Vendredi 09 mai 2025 à 20:45".to_string(),
        MatchNature::Rugby,
        None,
    );

    let mut e4 = Encounter::new(
        ClubType::UnionBordeauxBegles,
        "Union Bordeaux Bègles vs Clermont".to_string(),
        "Dimanche 27 avril 2025 à 15:00".to_string(),
        MatchNature::Rugby,
        Some("https://ticketing.ubb.rugby/resale/UBB-CLR-2025".to_string()),
    );
    e4.seats = Some(vec![
        make_seat("Virage Supporters", "VS-07", "D", 3, "28.00"),
        make_seat("Virage Supporters", "VS-07", "D", 4, "28.00"),
        make_seat("Tribune Présidentielle", "TP-01", "A", 1, "95.00"),
    ]);

    let e5 = Encounter::new(
        ClubType::UnionBordeauxBegles,
        "Union Bordeaux Bègles vs Castres Olympique".to_string(),
        "Samedi 10 mai 2025 à 17:30".to_string(),
        MatchNature::Rugby,
        None,
    );

    let e6 = Encounter::new(
        ClubType::StadeRochelais,
        "Meet & Greet joueurs - Stade Rochelais".to_string(),
        "Mercredi 30 avril 2025 à 18:00".to_string(),
        MatchNature::Other,
        None,
    );

    vec![e1, e2, e3, e4, e5, e6]
}

fn main() {
    let encounters = fake_encounters();

    // example_price_filter(&encounters);
    // example_filter_name(&encounters);
    // example_filter_chain_2(&encounters);
    // example_filter_chain(&encounters);
    // example_aggressive_chain(&encounters);
    example_rule_set(&encounters);
}


    // println!("{} rencontres parsées :\n", encounters.len());
    // for e in &encounters {
    //     let seat_count = e.seats.as_ref().map(|s| s.len()).unwrap_or(0);
    //     println!(
    //         "[{}] {:?} | {} | {} | resale: {} | {} place(s)",
    //         e.id,
    //         e.club_type,
    //         e.title,
    //         e.date,
    //         e.resale_link.as_deref().unwrap_or("—"),
    //         seat_count,
    //     );
    //     if let Some(seats) = &e.seats {
    //         for s in seats {
    //             println!(
    //                 "      {} — {}€",
    //                 s.seat_info.full_name,
    //                 s.price,
    //             );
    //         }
    //     }
    // }

#[allow(dead_code)]
fn example_price_filter(encounters: &Vec<Encounter>) {
    let filter = PriceFilter::new(Some(80.0), Some(100.0));
    let result = filter.apply(encounters);
    dbg!(&result);
}

#[allow(dead_code)]
fn example_filter_name(encounters: &Vec<Encounter>) {
    let filter = EncounterFilter::new(Some("Stade Rochelais vs Leinster Rugby".to_string()));
    let result = filter.apply(encounters);
    dbg!(&result);
}

#[allow(dead_code)]
fn example_filter_chain_2(encounters: &Vec<Encounter>) {
    let chain = FilterChain::new()
    .add(EncounterFilter::new(Some("Leinster Rugby".to_string())))
    .add(SeatPositionFilter::new(
        None,
        Some(2),
    ));
    let result = chain.apply(encounters);
    dbg!(&result);
}

#[allow(dead_code)]
fn example_filter_chain(encounters: &Vec<Encounter>) {
    let chain = FilterChain::new()
    .add(SeatPositionFilter::new(
        None,
        Some(2),
    ))
    .add(SeatPositionFilter::new(
        Some(SeatComposition {
            category: "Virage Supporters".to_string(),
            bloc: "VS-07".to_string(),
            row: "D".to_string(),
            seat_number: 3,
        }),
        None,
    ));
    
    let result = chain.apply(encounters);
    dbg!(&result);
}

#[allow(dead_code)]
fn example_rule_set(encounters: &Vec<Encounter>) {
    let rule_leinster = Rule::new(
        "Leinster — 1 place chère",
        FilterChain::new()
            .add(EncounterFilter::new(Some("Clermont".to_string())))
            .add(PriceFilter::new(Some(80.0), None)),
        |encounters: &[Encounter]| {
            println!("[Leinster] {} rencontre(s) trouvée(s) :", encounters.len());
            for e in encounters {
                println!("  → {} | {}", e.title, e.date);
                if let Some(seats) = &e.seats {
                    for s in seats {
                        println!("      {} — {}€", s.seat_info.full_name, s.price);
                    }
                }
            }
        },
    );

    let rule_toulouse = Rule::new(
        "Toulouse — prix max 40€",
        FilterChain::new()
            .add(EncounterFilter::new(Some("Stade Toulousain".to_string())))
            .add(PriceFilter::new(None, Some(40.0))),
        |encounters: &[Encounter]| {
            println!("[Toulouse] {} rencontre(s) trouvée(s) :", encounters.len());
            for e in encounters {
                println!("  → {} | {}", e.title, e.date);
                if let Some(seats) = &e.seats {
                    for s in seats {
                        println!("      {} — {}€", s.seat_info.full_name, s.price);
                    }
                }
            }
        },
    );

    let ruleset = RuleSet::new()
        .add(rule_leinster)
        .add(rule_toulouse);

    ruleset.run(encounters);
}