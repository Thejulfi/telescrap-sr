
use tokio::time::{interval, Duration};
use crate::core::scan::{ScanConfig, ScanResult};
use crate::controller::notify::Notify;
use crate::app::diff::diff;
use parser::interface::match_manager;

pub struct ScanTask<N: Notify> {
    config: ScanConfig,
    notifier: N,
    previous: Option<ScanResult>,
}

impl<N: Notify> ScanTask<N> {
    pub fn new(config: ScanConfig, notifier: N) -> Self {
        Self { config, notifier, previous: None }
    }

    pub async fn run(mut self) {
        let mut ticker = interval(Duration::from_secs(self.config.interval));
        loop {
            ticker.tick().await;
            let club = self.config.club.clone();
            let scan_result = tokio::task::spawn_blocking(move || {
                let encounters = match_manager::get_seats_from_rugby_matches(Some(club));
                ScanResult::new(encounters)
            })
            .await
            .unwrap();

            // Check if there is changes
            if let Some(prev) = &self.previous {
                let changed = diff(&prev.encounters, &scan_result.encounters);
                if changed.is_empty() {
                    println!("Aucun changement depuis le dernier scan.");
                } else {
                    self.notify_parsed_info(&scan_result, &changed);
                }
            } else {
                // First iteration: notify if at least one seat is available
                let encounters_with_seats: Vec<_> = scan_result.encounters.iter()
                    .filter(|e| e.seats.as_ref().map_or(false, |s| !s.is_empty()))
                    .cloned()
                    .collect();
                if !encounters_with_seats.is_empty() {
                    self.notify_parsed_info(&scan_result, &encounters_with_seats);
                } else {
                    self.notifier.send("Premier scan effectué, résultats enregistrés.");
                }
            }

            self.previous = Some(scan_result);
        }
    }

    fn notify_parsed_info(&self, scan_result: &ScanResult, changed: &[parser::core::encounter::Encounter]) {
        println!("Scanned at: {:?}", scan_result.scanned_at);
        let mut message = format!("Club : {}\n", self.config.club.name);
            for encounter in changed {
                let seat_list = match &encounter.seats {
                    Some(seats) if !seats.is_empty() => seats
                        .iter()
                        .map(|s| format!(
                            "  - {} ({})",
                            s.seat_info.as_deref().unwrap_or("?"),
                            s.price.as_deref().unwrap_or("prix inconnu")
                        ))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    _ => "  Aucun seat".to_string(),
                };
                message.push_str(&format!("\nMatch : {}\nSeats :\n{}", encounter.title, seat_list));
            }
            self.notifier.send(&message);
    }
}