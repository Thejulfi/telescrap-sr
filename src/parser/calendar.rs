use crate::parser::{MatchDetails, Parser};
use chrono::prelude::*;
use scraper::CaseSensitivity;
use scraper::{Html, Selector};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub calendar_sources: Vec<CalendarSource>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CalendarSource {
    pub name: String,
    pub url: String,
    pub enabled: bool,
}

pub fn load_config_from_str(content: &str) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let cfg: AppConfig = toml::from_str(content)?;
    Ok(cfg)
}

pub async fn get_calendar_from_url(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;
    response.error_for_status()?.text().await
}

pub async fn parse_calendar(
    url: &str,
    championship: &str,
) -> Result<Vec<MatchDetails>, Box<dyn std::error::Error>> {
    let body = get_calendar_from_url(url).await?;
    let document = Html::parse_document(body.as_str());

    let mut current_season_calendar: Vec<MatchDetails> = Vec::new();
    let journee = Selector::parse(".journee")?;
    let match_selector = Selector::parse(".matchs")?;
    let li_selector = Selector::parse("li")?;
    let a_with_title_selector = Selector::parse("a[title]")?;
    let result_selector = Selector::parse("div.fl.res.txtcenter")?;

    for item in document.select(&journee) {
        for match_item in item.select(&match_selector) {
            let mut date_text = "date not found".to_string();

            for li in match_item.select(&li_selector) {
                if li
                    .value()
                    .has_class("sep_dat", CaseSensitivity::AsciiCaseInsensitive)
                {
                    date_text = li.text().collect::<String>().trim().to_string();
                    continue;
                }

                let Some(a_tag) = li.select(&a_with_title_selector).next() else {
                    continue;
                };

                if !a_tag
                    .value()
                    .attr("title")
                    .unwrap_or("")
                    .contains("Stade Rochelais")
                {
                    continue;
                }

                let title = a_tag.value().attr("title").unwrap_or("title not found");
                let mut result_text = li
                    .select(&result_selector)
                    .next()
                    .map(|div| div.text().collect::<String>().trim().to_string())
                    .unwrap_or_else(|| "result not found".to_string());

                if result_text.contains(" - ") {
                    result_text = "match already played".to_string();
                }

                if result_text == "00h00" {
                    result_text = "match not programmed yet".to_string();
                }

                let parsed_date = Parser::parse_french_date(&date_text)
                    .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
                let parsed_hour = Parser::parse_french_time(&result_text);

                let timestamp_i64 = Parser::date_time_to_timestamp(parsed_date, parsed_hour);
                let timestamp = DateTime::<Utc>::from_timestamp(timestamp_i64, 0);

                current_season_calendar.push(MatchDetails {
                    championship: championship.to_string(),
                    city: "La Rochelle".to_string(),
                    date_human_readable: date_text.clone(),
                    date: parsed_date,
                    hour: parsed_hour,
                    timestamp,
                    match_title: title
                        .strip_prefix("Feuille de match : ")
                        .unwrap_or(title)
                        .to_string(),
                });
            }
        }
    }

    Ok(current_season_calendar)
}

pub fn next_upcoming_match(calendar: &[MatchDetails], now: DateTime<Utc>) -> Option<&MatchDetails> {
    calendar
        .iter()
        .filter_map(|m| m.timestamp.map(|ts| (m, ts)))
        .filter(|(_, ts)| *ts > now)
        .min_by_key(|(_, ts)| *ts)
        .map(|(m, _)| m)
}
