use std::collections::HashMap;
use scraper::{Html, Selector};
use serde::Deserialize;
use crate::core::seat::{Seat, SeatAction, SeatInfo, SeatComposition};
use crate::core::encounter::Encounter;
use crate::app::clubs::parsers::ParseSeat;

pub struct LarochellSeatParser;

#[derive(Deserialize)]
struct PackJson {
    pack_id: u64,
    tickets: HashMap<String, TicketInfo>,
    amount_by_ticket: String,
}

#[derive(Deserialize)]
struct TicketInfo {
    seat_info: String,
}

#[derive(Deserialize)]
struct DrupalSettings {
    #[serde(rename = "ajaxPageState")]
    ajax_page_state: AjaxPageState,
    ajax: HashMap<String, AjaxEntry>,
}

#[derive(Deserialize)]
struct AjaxPageState {
    libraries: String,
}

#[derive(Deserialize)]
struct AjaxEntry {
    url: String,
}

struct PageContext {
    ajax_url: String,
    libraries: String,
    price_min: String,
    price_max: String,
    form_build_id: String,
    form_token: String,
}

fn extract_page_context(document: &Html) -> Option<PageContext> {
    let script_selector = Selector::parse("script[data-drupal-selector='drupal-settings-json']").unwrap();
    let slider_selector = Selector::parse("input.hubber-slider-input").unwrap();
    let form_build_id_selector = Selector::parse("input[name='form_build_id']").unwrap();

    let settings_json = document.select(&script_selector).next()?.text().collect::<String>();
    let settings: DrupalSettings = serde_json::from_str(&settings_json).ok()?;

    let ajax_url = settings.ajax.get("edit-add-to-cart").map(|e| e.url.clone()).unwrap_or_default();
    let libraries = settings.ajax_page_state.libraries;

    let slider = document.select(&slider_selector).next()?;
    let price_min = slider.value().attr("data-min").unwrap_or("").to_string();
    let price_max = slider.value().attr("data-max").unwrap_or("").to_string();

    let form_build_id = document
        .select(&form_build_id_selector)
        .next()
        .and_then(|el| el.value().attr("value"))
        .unwrap_or("")
        .to_string();

    let form_token_selector = Selector::parse("input[name='form_token']").unwrap();
    let form_token = document
        .select(&form_token_selector)
        .next()
        .and_then(|el| el.value().attr("value"))
        .unwrap_or("")
        .to_string();

    Some(PageContext { ajax_url, libraries, price_min, price_max, form_build_id, form_token })
}

pub fn parse_seat(html: &str, _encounter: Encounter) -> Vec<Seat> {
    let document = Html::parse_document(html);

    let pack_selector = Selector::parse("div.js-listing-plain[data-json]").unwrap();
    let span_selector = Selector::parse("span").unwrap();

    let context = extract_page_context(&document);

    let mut seats = Vec::new();

    for pack_el in document.select(&pack_selector) {
        let data_json = match pack_el.value().attr("data-json") {
            Some(j) => j,
            None => continue,
        };
        let pack: PackJson = match serde_json::from_str(data_json) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to parse pack JSON: {}", e);
                continue;
            }
        };

        for (ticket_id, ticket) in &pack.tickets {
            let seat_doc = Html::parse_fragment(&ticket.seat_info);
            let seat_text = seat_doc
                .select(&span_selector)
                .next()
                .map(|el| el.text().collect::<String>())
                .unwrap_or_default();

            let actions = SeatAction {
                pack_id: pack.pack_id.to_string(),
                ticket_id: ticket_id.clone(),
                ajax_url: context.as_ref().map(|c| c.ajax_url.clone()).unwrap_or_default(),
                price_min: context.as_ref().map(|c| c.price_min.clone()).unwrap_or_default(),
                price_max: context.as_ref().map(|c| c.price_max.clone()).unwrap_or_default(),
                libraries: context.as_ref().map(|c| c.libraries.clone()).unwrap_or_default(),
                form_build_id: context.as_ref().map(|c| c.form_build_id.clone()).unwrap_or_default(),
                form_token: context.as_ref().map(|c| c.form_token.clone()).unwrap_or_default(),
            };

            seats.push(Seat {
                seat_info: if seat_text.is_empty() { None } else {
                    Some(SeatInfo {
                        full_name: seat_text.clone(),
                        composition: get_seat_composition(&seat_text.to_string()),
                    })
                },
                price: Some(pack.amount_by_ticket.clone()),
                actions,
            });
        }
    }

    seats
}

fn get_seat_composition(seat_info: &str) -> SeatComposition {
    let parts: Vec<&str> = seat_info.split('•').map(|s| s.trim()).collect();
    let mut composition = SeatComposition { access: "".to_string(), row: "".to_string(), seat_number: 0 };

    for part in parts {
        if part.starts_with("Accès") {
            composition.access = part.replacen("Accès", "", 1).trim().to_string();
        } else if part.starts_with("Rang") {
            composition.row = part.replacen("Rang", "", 1).trim().to_string();
        } else if part.starts_with("Siège") {
            if let Ok(num) = part.replacen("Siège", "", 1).trim().parse::<u64>() {
                composition.seat_number = num;
            }
        }
    }

    composition
}

impl ParseSeat for LarochellSeatParser {
    fn parse_seat(&self, html: &str, encounter: Encounter) -> Vec<Seat> {
        parse_seat(html, encounter)
    }
}
