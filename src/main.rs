#![deny(warnings)]
use scraper::{ElementRef, Html, Selector};
use std::fs;

#[derive(Debug)]
struct Match {
    title: String,
    is_resale: bool,
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    // Some simple CLI args requirements...
    let url = if let Some(url) = std::env::args().nth(1) {
        url
    } else {
        println!("No CLI URL provided, using default.");
        "https://billetterie.staderochelais.com/fr".into()
    };

    eprintln!("Fetching {url:?}...");

    let mut matchs: Vec<Match> = Vec::new();

    let res = reqwest::get(url).await?;

    let body = res.text().await?;

    fs::write("response.html", &body).expect("Unable to write file");

    let document = Html::parse_document(&body);

    let selector = Selector::parse(".actions-wrapper").unwrap();
    let resale_button_selector = Selector::parse("button.btn-resale").unwrap();
    let stade_link_selector = Selector::parse("a[href*=\"stade_rochelais\"]").unwrap();
    let h3_selector = Selector::parse("h3.title").unwrap();

    let mut count = 0;
    let mut extracted = String::new();
    for actions in document.select(&selector) {
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

            count += 1;
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

            matchs.push(Match {
                title: h3_text.clone(),
                is_resale: is_resale_available,
            });
            println!("Found match: {h3_text}, resale available: {is_resale_available}");

            extracted.push_str(&format!("<h3>{}</h3>\n", h3_text));
            extracted.push_str(&resale_button.html());
            extracted.push_str("\n\n");
        }
    }

    println!(
        "match name : {}, resale available: {}",
        matchs[0].title, matchs[0].is_resale
    );

    fs::write("extract.txt", extracted).expect("Unable to write extract.txt");
    eprintln!("Saved {count} actions-wrapper blocks to extract.txt");

    Ok(())
}

// The [cfg(not(target_arch = "wasm32"))] above prevent building the tokio::main function
// for wasm32 target, because tokio isn't compatible with wasm32.
// If you aren't building for wasm32, you don't need that line.
// The two lines below avoid the "'main' function not found" error when building for wasm32 target.
#[cfg(target_arch = "wasm32")]
fn main() {}
