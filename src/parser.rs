use std::collections::HashSet;

use scraper::{ElementRef, Html, Selector};

const FILTER_STADE_ROCHELAIS_ONLY: bool = true;

#[derive(Debug, Clone)]
pub struct Match {
    pub title: String,
    pub is_resale: bool,
}

pub struct Parser {
    pub url: String,
}

impl Parser {
    pub fn new(url: String) -> Self {
        Parser { url }
    }

    pub async fn fetch_and_parse(&self) -> Result<Vec<Match>, reqwest::Error> {
        let response = reqwest::get(&self.url).await?;
        let body = response.text().await?;

        let matches = Self::parse_matches(&body);

        let mut sorted_matches = Vec::new();

        sorted_matches = Self::sort_matches(&matches).unwrap_or_else(|err| {
            eprintln!("Error sorting matches: {err}");
            sorted_matches
        });

        Ok(sorted_matches)
    }

    fn parse_matches(body: &str) -> Vec<Match> {
        let mut matches = Vec::new();

        let document = Html::parse_document(body);
        let actions_selector = Selector::parse(".actions-wrapper").unwrap();
        let resale_action_selector = Selector::parse(".btn-resale").unwrap();
        let stade_link_selector =
            Selector::parse("a[href*=\"stade_rochelais\"]:not([href*=\"stade_rochelais_basket\"])")
                .unwrap();
        let h3_selector = Selector::parse("h3.title").unwrap();

        for actions in document.select(&actions_selector) {
            if let Some(resale_action) = actions.select(&resale_action_selector).next() {
                if FILTER_STADE_ROCHELAIS_ONLY
                    && actions.select(&stade_link_selector).next().is_none()
                {
                    continue;
                }

                let is_resale_available = resale_action.value().attr("href").is_some()
                    || resale_action
                        .value()
                        .attr("class")
                        .map(|class_attr| {
                            !class_attr
                                .split_whitespace()
                                .any(|class_name| class_name == "unavailable")
                        })
                        .unwrap_or(false);

                let h3_text = actions
                    .ancestors()
                    .filter_map(ElementRef::wrap)
                    .find_map(|ancestor| {
                        ancestor
                            .select(&h3_selector)
                            .next()
                            .map(|h3| h3.text().collect::<String>().trim().to_string())
                    })
                    .unwrap_or_else(|| "H3 not found".to_string());

                matches.push(Match {
                    title: h3_text,
                    is_resale: is_resale_available,
                });
            }
        }

        matches
    }

    fn sort_matches(matches: &[Match]) -> Result<Vec<Match>, String> {
        let mut sorted_matches = Vec::new();
        let mut seen_titles = HashSet::new();

        for match_item in matches.iter() {
            if match_item.is_resale && seen_titles.insert(match_item.title.clone()) {
                sorted_matches.push(match_item.clone());
            }
        }

        if sorted_matches.is_empty() {
            return Err("No matches with resale available".to_string());
        }

        Ok(sorted_matches)
    }
}
