mod calendar;

use chrono::prelude::*;
use regex::Regex;
use scraper::{Html, Selector};
use std::collections::HashSet;

const FILTER_STADE_ROCHELAIS_ONLY: bool = true;
const BASE_URL: &str = "https://billetterie.staderochelais.com";

#[derive(Debug, Clone)]
pub struct Match {
    pub title: String,
    pub is_resale: bool,
    pub url: Option<String>,
    pub date: String,
}

#[derive(Debug, Clone)]
pub struct Parser {
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct MatchDetails {
    pub championship: String,
    #[allow(dead_code)]
    pub city: String,
    pub date_human_readable: String,
    #[allow(dead_code)]
    pub date: NaiveDate,
    pub hour: Option<NaiveTime>,
    pub timestamp: Option<DateTime<Utc>>,
    pub match_title: String,
}

impl Parser {
    pub fn new(url: String) -> Self {
        Parser { url }
    }

    pub async fn next_upcoming_match_from_urls(
        &self,
        sources: &[(&str, &str)],
        now: DateTime<Utc>,
    ) -> Option<MatchDetails> {
        let mut calendars = Vec::new();

        for (url, championship) in sources {
            match calendar::parse_calendar(url, championship).await {
                Ok(calendar) => calendars.extend(calendar),
                Err(err) => {
                    crate::log::error(format!("Error while parsing calendar page {url}: {err}"));
                }
            }
        }

        calendar::next_upcoming_match(&calendars, now).cloned()
    }

    pub async fn fetch_and_parse(&self) -> Result<Vec<Match>, reqwest::Error> {
        let response = reqwest::get(&self.url).await?;
        let body = response.text().await?;

        let matches = Self::parse_matches(&body).await?;

        let mut sorted_matches = Vec::new();

        sorted_matches = Self::sort_matches(&matches).unwrap_or_else(|err| {
            crate::log::error(format!("Error sorting matches: {err}"));
            sorted_matches
        });

        Ok(sorted_matches)
    }

    async fn parse_matches(body: &str) -> Result<Vec<Match>, reqwest::Error> {
        let mut matches = Vec::new();

        let document = Html::parse_document(body);
        let items = Selector::parse(".field__item").unwrap();
        let resale_action_selector = Selector::parse(".btn-resale").unwrap();
        let stade_link_selector =
            Selector::parse("a[href*=\"stade_rochelais\"]:not([href*=\"stade_rochelais_basket\"])")
                .unwrap();
        let h3_selector = Selector::parse("h3.title").unwrap();
        let date_selector = Selector::parse("a.sale-btn").unwrap();
        let re = Regex::new(
            r"(lundi|mardi|mercredi|jeudi|vendredi|samedi|dimanche)_(\d{1,2})_(\w+)_(\d{4})",
        )
        .unwrap();

        for item in document.select(&items) {
            if let Some(resale_action) = item.select(&resale_action_selector).next() {
                if FILTER_STADE_ROCHELAIS_ONLY && item.select(&stade_link_selector).next().is_none()
                {
                    continue;
                }

                let match_title = item
                    .select(&h3_selector)
                    .next()
                    .map(|h3| h3.text().collect::<String>().trim().to_string())
                    .unwrap_or_else(|| "H3 not found".to_string());

                if matches.iter().any(|m: &Match| m.title == match_title) {
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

                let raw_date = item
                    .select(&date_selector)
                    .next()
                    .and_then(|date| date.value().attr("href"))
                    .map(|href| href.to_string())
                    .unwrap_or_else(|| "Date not found".to_string());

                let Some(match_date) = re.captures(&raw_date) else {
                    crate::log::error(format!("Date format not recognized: {raw_date}"));
                    continue;
                };

                matches.push(Match {
                    title: match_title,
                    is_resale: is_resale_available,
                    url: resale_action
                        .value()
                        .attr("href")
                        .map(Self::build_absolute_url),
                    date: format!(
                        "{} {} {} {}",
                        &match_date[1], &match_date[2], &match_date[3], &match_date[4]
                    ),
                });
            }
        }

        Ok(matches)
    }

    pub(crate) fn parse_french_time(time_str: &str) -> Option<NaiveTime> {
        // format: "21h00"
        NaiveTime::parse_from_str(time_str, "%Hh%M").ok()
    }

    pub(crate) fn parse_french_date(date_str: &str) -> Option<NaiveDate> {
        let parts: Vec<&str> = date_str.split_whitespace().collect();
        // format: "samedi 12 octobre 2024"
        //            [0]   [1]    [2]    [3]
        if parts.len() != 4 {
            return None;
        }

        let day: u32 = parts[1].parse().ok()?;
        let year: i32 = parts[3].parse().ok()?;
        let month: u32 = match parts[2].to_lowercase().as_str() {
            "janvier" => 1,
            "février" => 2,
            "mars" => 3,
            "avril" => 4,
            "mai" => 5,
            "juin" => 6,
            "juillet" => 7,
            "août" => 8,
            "septembre" => 9,
            "octobre" => 10,
            "novembre" => 11,
            "décembre" => 12,
            _ => return None,
        };

        NaiveDate::from_ymd_opt(year, month, day)
    }

    fn build_absolute_url(href: &str) -> String {
        if href.starts_with("http://") || href.starts_with("https://") {
            href.to_string()
        } else if href.starts_with('/') {
            format!("{BASE_URL}{href}")
        } else {
            format!("{BASE_URL}/{href}")
        }
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

    pub(crate) fn date_time_to_timestamp(date: NaiveDate, hour: Option<NaiveTime>) -> i64 {
        let time = hour.unwrap_or_else(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        let naive_datetime = date.and_time(time);
        naive_datetime.and_utc().timestamp()
    }
}
