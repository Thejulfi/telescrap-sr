
use parser::core::encounter::Encounter;
/// This module defines the `ScanTask` struct and its associated logic for performing periodic scans of encounters,
/// applying filters, and notifying about changes in available seats.
use parser::interface::match_manager;
use filter::filter::Filter;
use tokio::sync::watch;
use tokio::time::{interval, Duration};
use crate::{
    app::diff::{diff, DiffType},
    controller::notify::Notify,
    core::scan::{ScanConfig, ScanMode, ScanResult},
};


/// Represents a scanning task that periodically checks for changes in encounters based on a specified configuration,
/// applies filters to the results, and sends notifications about any detected changes.
pub struct ScanTask<N: Notify> {
    config: ScanConfig,
    config_rx: watch::Receiver<ScanConfig>,
    notifier: N,
    previous: Option<ScanResult>,
}

impl<N: Notify> ScanTask<N> {
    /// Creates a new `ScanTask` with the specified configuration and notifier.
    ///
    /// # Arguments
    /// * `config` - The configuration for the scan task.
    /// * `notifier` - The notifier to use for sending notifications about changes.
    ///
    /// # Returns
    /// A new instance of `ScanTask` initialized with the provided configuration and notifier.
    pub fn new(mut config_rx: watch::Receiver<ScanConfig>, notifier: N) -> Self {
        let config = config_rx.borrow_and_update().clone();
        Self { config, config_rx, notifier, previous: None }
    }

    /// Runs the scan task, periodically checking for changes in encounters, applying filters,
    /// and sending notifications about any detected changes.
    pub async fn run(mut self) {
        let mut ticker = interval(Duration::from_secs(self.config.interval));
        loop {
            tokio::select! {
                _ = ticker.tick() => {}
                result = self.config_rx.changed() => {
                    if result.is_err() {
                        break; // sender dropped (shutdown), exit cleanly
                    }
                    self.config = self.config_rx.borrow_and_update().clone();
                    self.previous = None;
                    ticker = interval(Duration::from_secs(self.config.interval));
                    println!("⚙️  Configuration mise à jour, redémarrage du cycle");
                    continue;
                }
            }
            // variable that will be used to measure the duration of the scan process
            let scan_start = std::time::Instant::now();
            // Fetch current encounters from the match manager in a blocking task to avoid blocking the async runtime
            let club = self.config.club.clone();
            // Fetch match nature from config
            let nature = self.config.nature;

            // Fetch encounters from the match manager in a blocking task to avoid blocking the async runtime
            let scan_result = tokio::task::spawn_blocking(move || {
                let encounters = match_manager::get_seats_from_matches(club, nature);
                println!("📋 {} rencontre(s) récupérée(s)", encounters.len());
                ScanResult::new(encounters)
            })
            .await
            .unwrap();

        // Compare with previous results to detect changes (new seats)
        let changed: Vec<Encounter> = if let Some(prev) = &self.previous {
            diff(&prev.encounters, &scan_result.encounters)
                .into_iter()
                .filter(|r| r.diff_type == DiffType::NewSeats)
                .map(|r| r.encounter_diff_only)
                .collect()
        } else {
            // First iteration: treat available seats as new
            scan_result.encounters.iter()
                .filter(|e| e.seats.as_ref().map_or(false, |s| !s.is_empty()))
                .cloned()
                .collect()
        };
        
        // Apply the filter chain from config (built by admin panel)
        let result = if let Some(chain) = &self.config.filter_chain {
            chain.apply(&changed)
        } else {
            changed
        };

        // Load seat preview images if enabled
        let mut result = result;
        if self.config.is_preview {
            for encounter in &mut result {
                if let Some(seats) = &mut encounter.seats {
                    for seat in seats.iter_mut() {
                        seat.seat_info.preview_url = match_manager::get_seat_preview(
                            &self.config.club,
                            &seat.seat_info.composition,
                        );
                    }
                }
            }
        }

        // Calculate elapsed time and send notifications if there are changes, otherwise log that no change was detected
        if !result.is_empty() {
                let elapsed = scan_start.elapsed();
                println!("⏱️  Scan terminé en {:.2?} ({} match(es) trouvé(s))", elapsed, result.len());
                self.notify_parsed_info(&result);
            } else {
                let elapsed = scan_start.elapsed();
                println!("✅ Aucun changement détecté ({:.2?})", elapsed);
            }

            self.previous = Some(scan_result);
        }
    }

    /// Notifies about the parsed information by constructing a message that includes the details of the encounters and the detected changes, and sending it through the notifier.
    /// 
    /// # Arguments
    /// * `scan_result` - The result of the scan containing the list of encounters and the timestamp of when the scan was performed.
    /// * `changed` - A slice of `DiffResult` instances representing the detected changes that should be included in the notification.
    ///
    /// # Returns
    /// This method does not return a value, but it sends a formatted message through the notifier containing the details of the encounters and the detected changes.
    fn notify_parsed_info(&self, changed: &[Encounter]) {
        let header = format!("🏉 <b>{}</b>", self.config.club.name);

        for encounter in changed {
            let resale = match &encounter.resale_link {
                Some(link) => format!("\n🔗 <a href=\"{}\">Accéder à la revente</a>", link),
                None => String::new(),
            };

            let encounter_header = format!(
                "{}\n\n━━━━━━━━━━━━━━━━\n\n🆚 <b>{}</b>\n📅 <i>{}</i>\n\n🟢 <b>Nouvelles places :</b>{}",
                header, encounter.title, encounter.date, resale,
            );

            match &encounter.seats {
                Some(seats) if !seats.is_empty() => {
                    // Seats with a preview: one photo message per seat
                    let (with_preview, without_preview): (Vec<_>, Vec<_>) = seats.iter()
                        .partition(|s| s.seat_info.preview_url.is_some());

                    for seat in &with_preview {
                        let category = seat.seat_info.composition.category.as_str();
                        let full_name = seat.seat_info.full_name.as_str();
                        let price = seat.price.as_str();
                        let seat_line = if category.is_empty() {
                            format!("  • {} — <code>{}€ </code>", full_name, price)
                        } else {
                            format!("  • [{}] {} — <code>{}€ </code>", category, full_name, price)
                        };
                        let caption = format!("{}\n\n{}", encounter_header, seat_line);
                        self.notifier.send_photo(seat.seat_info.preview_url.as_deref().unwrap(), &caption);
                    }

                    // Remaining seats without preview: one grouped text message
                    if !without_preview.is_empty() {
                        let seat_list = without_preview.iter()
                            .map(|s| {
                                let category = s.seat_info.composition.category.as_str();
                                let full_name = s.seat_info.full_name.as_str();
                                let price = s.price.as_str();
                                if category.is_empty() {
                                    format!("  • {} — <code>{}€ </code>", full_name, price)
                                } else {
                                    format!("  • [{}] {} — <code>{}€ </code>", category, full_name, price)
                                }
                            })
                            .collect::<Vec<_>>()
                            .join("\n");
                        self.notifier.send(&format!("{}\n\n{}", encounter_header, seat_list));
                    }
                }
                _ => {
                    self.notifier.send(&format!("{}\n\n  <i>Aucun siège disponible</i>", encounter_header));
                }
            }
        }
    }

    // TODO : see if that really useful
    // fn get_scanner_time_str(&self, scanned_at: &ScanResult) -> String {
    //     let secs = scan_result.scanned_at
    //         .duration_since(std::time::UNIX_EPOCH)
    //         .map(|d| d.as_secs())
    //         .unwrap_or(0);
    //     let (h, m, s) = (secs % 86400 / 3600, secs % 3600 / 60, secs % 60);
    //     let days_since_epoch = secs / 86400;
    //     // Simple Gregorian date from days since 1970-01-01
    //     let (mut y, mut doy) = (1970u32, days_since_epoch as u32);
    //     loop {
    //         let dy = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 366 } else { 365 };
    //         if doy < dy { break; }
    //         doy -= dy;
    //         y += 1;
    //     }
    //     let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    //     let months = [31u32, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    //     let (mut mo, mut d) = (1u32, doy + 1);
    //     for days_in_month in months {
    //         if d <= days_in_month { break; }
    //         d -= days_in_month;
    //         mo += 1;
    //     }
    //     format!("{:02}/{:02}/{} à {:02}:{:02}:{:02}", d, mo, y, h, m, s)
    // }
}