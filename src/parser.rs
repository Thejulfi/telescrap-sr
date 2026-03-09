use scraper::{ElementRef, Html, Selector};

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

        let mut matches = Self::parse_matches(&body);

        let mut sorted_matches = Vec::new();

        sorted_matches = Self::sort_matches(&mut matches).unwrap_or_else(|err| {
            eprintln!("Error sorting matches: {err}");
            sorted_matches
        });

        Ok(sorted_matches)
    }

    fn parse_matches(body: &str) -> Vec<Match> {
        let mut matches = Vec::new();

        let document = Html::parse_document(body);
        let actions_selector = Selector::parse(".actions-wrapper").unwrap();
        let resale_button_selector = Selector::parse("button.btn-resale").unwrap();
        let stade_link_selector = Selector::parse("a[href*=\"stade_rochelais\"]").unwrap();
        let h3_selector = Selector::parse("h3.title").unwrap();

        for actions in document.select(&actions_selector) {
            if let Some(resale_button) = actions.select(&resale_button_selector).next() {
                if actions.select(&stade_link_selector).next().is_none() {
                    continue;
                }

                let is_resale_available = resale_button
                    .value()
                    .attr("class")
                    .map(|class_attr| {
                        class_attr
                            .split_whitespace()
                            .any(|class_name| class_name == "available")
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

    fn sort_matches(matches: &mut Vec<Match>) -> Result<Vec<Match>, String> {
        let mut sorted_matches = Vec::new();

        for match_item in matches.iter_mut() {
            if match_item.is_resale {
                sorted_matches.push(match_item.clone());
            }
        }

        if sorted_matches.is_empty() {
            return Err("No matches with resale available".to_string());
        }

        Ok(sorted_matches)
    }
}
