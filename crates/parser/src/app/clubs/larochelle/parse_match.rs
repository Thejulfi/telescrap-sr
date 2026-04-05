use std::collections::HashSet;
use scraper::{Html, Selector};
use crate::core::encounter::{Encounter, MatchNature};
use crate::core::club::ClubType;
use crate::app::clubs::parsers::ParseMatch;

pub struct LarochellMatchParser;

pub fn parse_match(html: &str) -> Vec<Encounter> {
    let document = Html::parse_document(html);

    let meeting_selector = Selector::parse("div.meeting").unwrap();
    let title_selector = Selector::parse("h3.title").unwrap();
    let date_selector = Selector::parse("span.date").unwrap();
    let link_selector = Selector::parse("a.btn-resale").unwrap();

    let mut encounters = Vec::new();

    for meeting in document.select(&meeting_selector) {
        let title = meeting
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let date = meeting
            .select(&date_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        if !title.is_empty() {
            let nature = MatchNature::from_title(&title);
            let resale_link = meeting
                .select(&link_selector)
                .next()
                .and_then(|el| el.value().attr("href"))
                .map(|href| format!("https://billetterie.staderochelais.com/{}", href.trim_start_matches('/')));
            encounters.push(Encounter::new(ClubType::StadeRochelais, title, date, nature, resale_link));
        }
    }

    let mut seen = HashSet::new();
    encounters.retain(|e| seen.insert((e.title.clone(), e.date.clone())));

    encounters
}

impl ParseMatch for LarochellMatchParser {
    fn parse_match(&self, html: &str) -> Vec<Encounter> {
        parse_match(html)
    }
}