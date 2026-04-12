use axum::{
    Form,
    Router,
    extract::State,
    response::Html,
    routing::{get, post},
};
use parser::core::encounter::MatchNature;
use scanner::core::scan::{ScanConfig, ScanFilter, ScanMode};
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
    price_threshold: Option<String>,
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
    let filter = config.filter.as_ref();

    let sel_passive    = if config.mode == ScanMode::PassiveScan    { "selected" } else { "" };
    let sel_aggressive = if config.mode == ScanMode::AggressiveScan { "selected" } else { "" };
    let sel_rugby      = if config.nature == MatchNature::Rugby      { "selected" } else { "" };
    let sel_basketball = if config.nature == MatchNature::Basketball { "selected" } else { "" };
    let sel_other      = if config.nature == MatchNature::Other      { "selected" } else { "" };

    let price_threshold = filter.and_then(|f| f.price_threshold).map(|v| v.to_string()).unwrap_or_default();
    let side_by_side    = filter.and_then(|f| f.side_by_side).map(|v| v.to_string()).unwrap_or_default();
    let match_title     = filter.and_then(|f| f.match_title.as_deref()).unwrap_or("").to_string();
    let chk_preview     = if filter.and_then(|f| f.is_preview).unwrap_or(false) { "checked" } else { "" };

    let html = INDEX_HTML
        .replace("{interval}", &interval.to_string())
        .replace("{sel_passive}", sel_passive)
        .replace("{sel_aggressive}", sel_aggressive)
        .replace("{sel_rugby}", sel_rugby)
        .replace("{sel_basketball}", sel_basketball)
        .replace("{sel_other}", sel_other)
        .replace("{price_threshold}", &price_threshold)
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

    let price_threshold = form.price_threshold.filter(|s| !s.is_empty()).and_then(|s| s.parse::<f64>().ok());
    let side_by_side    = form.side_by_side.filter(|s| !s.is_empty()).and_then(|s| s.parse::<u64>().ok());
    let match_title     = form.match_title.filter(|s| !s.is_empty());
    let is_preview      = Some(form.is_preview.is_some());

    let existing = new_config.filter.take();
    new_config.filter = Some(ScanFilter {
        price_threshold,
        side_by_side,
        match_title,
        is_preview,
        date_range: existing.as_ref().and_then(|f| f.date_range.clone()),
        position: existing.as_ref().and_then(|f| f.position.clone()),
    });

    println!("[DEBUG] Config mise à jour : interval={}s, mode={:?}, nature={:?} side_by_side={:?}", new_config.interval, new_config.mode, new_config.nature, new_config.filter.as_ref().and_then(|f| f.side_by_side));
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