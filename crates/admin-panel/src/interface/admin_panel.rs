use axum::{
    Form,
    Router,
    extract::State,
    response::Html,
    routing::{get, post},
};
use filter::filter::filter_chain::FilterChain;
use filter::filter::config::{
    encounter::EncounterFilter,
    price::PriceFilter,
    seat::SeatPositionFilter,
};
use parser::core::encounter::MatchNature;
use scanner::core::scan::{ScanConfig, ScanMode};
use serde::Deserialize;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::watch;

const INDEX_HTML: &str = include_str!("pages/index.html");
const CONFIG_UPDATED_HTML: &str = include_str!("pages/config_updated.html");

#[derive(Clone)]
struct AppState {
    config_tx: Arc<watch::Sender<ScanConfig>>,
}

#[derive(Deserialize)]
struct ScanConfigForm {
    interval: u64,
    mode: String,
    nature: String,
    #[serde(default)]
    price_min: Option<String>,
    #[serde(default)]
    price_max: Option<String>,
    #[serde(default)]
    seat_category: Option<String>,
    #[serde(default)]
    seat_bloc: Option<String>,
    #[serde(default)]
    seat_row: Option<String>,
    #[serde(default)]
    side_by_side: Option<String>,
    #[serde(default)]
    match_title: Option<String>,
    #[serde(default)]
    is_preview: Option<String>,
}

async fn index(State(state): State<AppState>) -> Html<String> {
    let config = state.config_tx.borrow();
    let interval = config.interval;

    let sel_passive    = if config.mode == ScanMode::PassiveScan    { "selected" } else { "" };
    let sel_aggressive = if config.mode == ScanMode::AggressiveScan { "selected" } else { "" };
    let sel_rugby      = if config.nature == MatchNature::Rugby      { "selected" } else { "" };
    let sel_basketball = if config.nature == MatchNature::Basketball { "selected" } else { "" };
    let sel_other      = if config.nature == MatchNature::Other      { "selected" } else { "" };

    let chain           = config.filter_chain.as_deref();
    let price_min       = chain.and_then(|c| c.price_min()).map(|v| v.to_string()).unwrap_or_default();
    let price_max       = chain.and_then(|c| c.price_max()).map(|v| v.to_string()).unwrap_or_default();
    let seat_category   = chain.and_then(|c| c.seat_category()).unwrap_or("").to_string();
    let seat_bloc       = chain.and_then(|c| c.seat_bloc()).unwrap_or("").to_string();
    let seat_row        = chain.and_then(|c| c.seat_row()).unwrap_or("").to_string();
    let side_by_side    = chain.and_then(|c| c.side_by_side()).map(|v| v.to_string()).unwrap_or_default();
    let match_title     = chain.and_then(|c| c.encounter_title()).unwrap_or("").to_string();
    let chk_preview     = if config.is_preview { "checked" } else { "" };

    let html = INDEX_HTML
        .replace("{interval}", &interval.to_string())
        .replace("{sel_passive}", sel_passive)
        .replace("{sel_aggressive}", sel_aggressive)
        .replace("{sel_rugby}", sel_rugby)
        .replace("{sel_basketball}", sel_basketball)
        .replace("{sel_other}", sel_other)
        .replace("{price_min}", &price_min)
        .replace("{price_max}", &price_max)
        .replace("{seat_category}", &seat_category)
        .replace("{seat_bloc}", &seat_bloc)
        .replace("{seat_row}", &seat_row)
        .replace("{side_by_side}", &side_by_side)
        .replace("{match_title}", &match_title)
        .replace("{chk_preview}", chk_preview);

    Html(html)
}

async fn update_config(
    State(state): State<AppState>,
    Form(form): Form<ScanConfigForm>,
) -> Html<String> {
    let mut new_config = state.config_tx.borrow().clone();

    new_config.interval = form.interval;
    new_config.mode = match form.mode.as_str() {
        "aggressive" => ScanMode::AggressiveScan,
        _ => ScanMode::PassiveScan,
    };
    new_config.nature = match form.nature.as_str() {
        "basketball" => MatchNature::Basketball,
        "other" => MatchNature::Other,
        _ => MatchNature::Rugby,
    };

    let price_min       = form.price_min.filter(|s| !s.is_empty()).and_then(|s| s.parse::<f64>().ok());
    let price_max       = form.price_max.filter(|s| !s.is_empty()).and_then(|s| s.parse::<f64>().ok());
    let seat_category   = form.seat_category.filter(|s| !s.is_empty());
    let seat_bloc       = form.seat_bloc.filter(|s| !s.is_empty());
    let seat_row        = form.seat_row.filter(|s| !s.is_empty());
    let side_by_side    = form.side_by_side.filter(|s| !s.is_empty()).and_then(|s| s.parse::<u64>().ok());
    let match_title     = form.match_title.filter(|s| !s.is_empty());

    let position = if seat_category.is_some() || seat_bloc.is_some() || seat_row.is_some() {
        Some(parser::core::seat::SeatComposition {
            category: seat_category.clone().unwrap_or_default(),
            bloc: seat_bloc.clone().unwrap_or_default(),
            row: seat_row.clone().unwrap_or_default(),
            seat_number: 0,
        })
    } else {
        None
    };

    // new_config.match_title = match_title.clone();
    new_config.is_preview = form.is_preview.is_some();

    // Build the FilterChain from the form values
    let mut chain = FilterChain::new();
    if let Some(title) = match_title {
        chain = chain.add(EncounterFilter::new(Some(title)));
    }
    if price_min.is_some() || price_max.is_some() {
        chain = chain.add(PriceFilter::new(price_min, price_max));
    }
    if position.is_some() || side_by_side.is_some() {
        chain = chain.add(SeatPositionFilter::new(
            position,
            side_by_side.map(|n| n as usize),
        ));
    }
    new_config.filter_chain = Some(Arc::new(chain));

    println!("[DEBUG] Config mise à jour : interval={}s, nature={:?}", new_config.interval, new_config.nature);
    state.config_tx.send(new_config).ok();

    Html(CONFIG_UPDATED_HTML.to_string())
}

pub async fn run(config_tx: watch::Sender<ScanConfig>) {
    let state = AppState { config_tx: Arc::new(config_tx) };

    let app = Router::new()
        .route("/", get(index))
        .route("/config", post(update_config))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Serveur démarré sur http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}