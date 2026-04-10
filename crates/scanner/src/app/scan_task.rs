
/// This module defines the `ScanTask` struct and its associated logic for performing periodic scans of encounters,
/// applying filters, and notifying about changes in available seats.
use parser::interface::match_manager;
use tokio::time::{interval, Duration};
use crate::{
    app::diff::{diff, DiffResult, DiffType},
    controller::notify::Notify,
    core::scan::{ScanConfig, ScanMode, ScanResult},
};

/// Represents a scanning task that periodically checks for changes in encounters based on a specified configuration,
/// applies filters to the results, and sends notifications about any detected changes.
pub struct ScanTask<N: Notify> {
    config: ScanConfig,
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
    pub fn new(config: ScanConfig, notifier: N) -> Self {
        Self { config, notifier, previous: None }
    }

    /// Runs the scan task, periodically checking for changes in encounters, applying filters,
    /// and sending notifications about any detected changes.
    pub async fn run(mut self) {
        let mut ticker = interval(Duration::from_secs(self.config.interval));
        loop {
            ticker.tick().await;
            let scan_start = std::time::Instant::now();
            let club = self.config.club.clone();
            let has_filter = self.config.filter.is_some();
            let is_aggressive = self.config.mode == ScanMode::AggressiveScan;
            let match_title = self.config.filter.as_ref().and_then(|f| f.match_title.clone());
            let nature = self.config.nature;
            let scan_result = tokio::task::spawn_blocking(move || {

                let encounters = if match_title.is_some() {
                    if let Some(title) = match_title {
                        match_manager::get_seats_from_match_title(title, club, nature)
                    } else {
                        vec![]
                    }
                } else {
                    match_manager::get_seats_from_matches(club, nature)
                };
                if has_filter && is_aggressive {
                    println!("⚠️  Mode agressif activé, mise au panier en automatique des sièges disponibles");
                }
                ScanResult::new(encounters)
            })
            .await
            .unwrap();

            let changed: Vec<DiffResult> = if let Some(prev) = &self.previous {
                diff(&prev.encounters, &scan_result.encounters)
            } else {
                // First iteration: treat available seats as new
                scan_result.encounters.iter()
                    .filter(|e| e.seats.as_ref().map_or(false, |s| !s.is_empty()))
                    .map(|e| DiffResult { diff_type: DiffType::NewSeats, encounter_diff_only: e.clone() })
                    .collect()
            };

            // Apply side by side filter if enabled
            let changed = self.apply_side_by_side_filter(changed);
            // Apply price filter if enabled
            let changed = self.apply_price_filter(changed);
            // Apply seat position filter if enabled
            let changed = self.apply_position_filter(changed);
            // Only notify on new seats, not on removals
            let changed: Vec<DiffResult> = changed.into_iter().filter(|r| r.diff_type == DiffType::NewSeats).collect();

            // Check if we need to load seat's preview image for the notification
            let load_preview = self.config.filter.as_ref()
                .and_then(|f| f.is_preview)
                .unwrap_or(false);
            let mut changed = changed;
            if load_preview {
                for result in &mut changed {
                    if let Some(seats) = &mut result.encounter_diff_only.seats {
                        for seat in seats.iter_mut() {
                            seat.seat_info.preview_url = match_manager::get_seat_preview(
                                &self.config.club,
                                &seat.seat_info.composition,
                            );
                        }
                    }
                }
            }

            if !changed.is_empty() {
                let elapsed = scan_start.elapsed();
                println!("⏱️  Scan terminé en {:.2?} ({} match(es) trouvé(s))", elapsed, changed.len());
                self.notify_parsed_info(&changed);
            }

            self.previous = Some(scan_result);
        }
    }

    /// Applies the seat position filter to the list of detected changes, filtering out seats that do not match the specified criteria.
    /// 
    /// # Arguments
    /// * `changed` - A vector of `DiffResult` instances representing the detected changes before applying the position filter.
    /// 
    /// # Returns
    /// A vector of `DiffResult` instances representing the detected changes after applying the position filter.
    fn apply_position_filter(&self, changed: Vec<DiffResult>) -> Vec<DiffResult> {
        let pos_filter = match self.config.filter.as_ref().and_then(|f| f.position.clone()) {
            Some(p) => p,
            None => return changed,
        };

        changed.into_iter()
            .filter_map(|mut result| {
                let seats = match result.encounter_diff_only.seats.take() {
                    Some(s) if !s.is_empty() => s,
                    _ => return None,
                };

                let filtered: Vec<_> = seats.into_iter()
                    .filter(|s| {
                        let c = &s.seat_info.composition;
                        (pos_filter.category.is_empty() || c.category.to_lowercase().contains(&pos_filter.category.to_lowercase()))
                            && (pos_filter.bloc.is_empty() || c.bloc.to_lowercase().contains(&pos_filter.bloc.to_lowercase()))
                            && (pos_filter.row.is_empty() || c.row.to_lowercase() == pos_filter.row.to_lowercase())
                    })
                    .collect();

                if filtered.is_empty() {
                    None
                } else {
                    result.encounter_diff_only.seats = Some(filtered);
                    Some(result)
                }
            })
            .collect()
    }

    /// Applies the price filter to the list of detected changes, filtering out seats that exceed the specified price threshold.
    /// 
    /// # Arguments
    /// * `changed` - A vector of `DiffResult` instances representing the detected changes before applying the price filter.
    /// 
    /// # Returns
    /// A vector of `DiffResult` instances representing the detected changes after applying the price filter
    fn apply_price_filter(&self, changed: Vec<DiffResult>) -> Vec<DiffResult> {
        let threshold = match self.config.filter.as_ref().and_then(|f| f.price_threshold) {
            Some(t) => t,
            None => return changed,
        };

        changed.into_iter()
            .filter_map(|mut result| {
                let seats = match result.encounter_diff_only.seats.take() {
                    Some(s) if !s.is_empty() => s,
                    _ => return None,
                };

                let filtered: Vec<_> = seats.into_iter()
                    .filter(|s| s.price.trim_end_matches('€').parse::<f64>().ok().map_or(false, |p| p <= threshold))
                    .collect();

                if filtered.is_empty() {
                    None
                } else {
                    result.encounter_diff_only.seats = Some(filtered);
                    Some(result)
                }
            })
            .collect()
    }

    /// Applies the side-by-side filter to the list of detected changes, filtering out seats that do not have the required number of consecutive seats available.
    /// 
    /// # Arguments
    /// * `changed` - A vector of `DiffResult` instances representing the detected changes before applying the side-by-side filter.
    /// 
    /// # Returns
    /// A vector of `DiffResult` instances representing the detected changes after applying the side-by-side filter.
    fn apply_side_by_side_filter(&self, changed: Vec<DiffResult>) -> Vec<DiffResult> {
        let required = match self.config.filter.as_ref().and_then(|f| f.side_by_side) {
            Some(n) if n > 1 => n as usize,
            _ => return changed,
        };

        changed
            .into_iter()
            .filter_map(|mut result| {
                let seats = match result.encounter_diff_only.seats.take() {
                    Some(s) if !s.is_empty() => s,
                    _ => return None,
                };

                // Sort by (access, row, seat_number) so consecutive seats are adjacent
                let mut sorted = seats;
                sorted.sort_by(|a, b| {
                    let ac = Some((a.seat_info.composition.bloc.as_str(), a.seat_info.composition.row.as_str(), a.seat_info.composition.seat_number));
                    let bc = Some((b.seat_info.composition.bloc.as_str(), b.seat_info.composition.row.as_str(), b.seat_info.composition.seat_number));
                    ac.partial_cmp(&bc).unwrap_or(std::cmp::Ordering::Equal)
                });

                // Find indices that belong to a run of >= required consecutive seats
                let mut in_group = vec![false; sorted.len()];
                let mut run_start = 0;
                for i in 1..=sorted.len() {
                    let consecutive = i < sorted.len() && {
                        let cur = &sorted[i].seat_info.composition;
                        let prev = &sorted[i - 1].seat_info.composition;
                        cur.bloc == prev.bloc
                            && cur.row == prev.row
                            && cur.seat_number == prev.seat_number + 1
                    };
                    if !consecutive {
                        if i - run_start >= required {
                            for j in run_start..i {
                                in_group[j] = true;
                            }
                        }
                        run_start = i;
                    }
                }

                let filtered: Vec<_> = sorted.into_iter().enumerate()
                    .filter(|(i, _)| in_group[*i])
                    .map(|(_, s)| s)
                    .collect();

                if filtered.is_empty() {
                    None
                } else {
                    result.encounter_diff_only.seats = Some(filtered);
                    Some(result)
                }
            })
            .collect()
    }

    /// Notifies about the parsed information by constructing a message that includes the details of the encounters and the detected changes, and sending it through the notifier.
    /// 
    /// # Arguments
    /// * `scan_result` - The result of the scan containing the list of encounters and the timestamp of when the scan was performed.
    /// * `changed` - A slice of `DiffResult` instances representing the detected changes that should be included in the notification.
    ///
    /// # Returns
    /// This method does not return a value, but it sends a formatted message through the notifier containing the details of the encounters and the detected changes.
    fn notify_parsed_info(&self, changed: &[DiffResult]) {
        let header = format!("🏉 <b>{}</b>", self.config.club.name);

        for result in changed {
            let encounter = &result.encounter_diff_only;
            let (status_icon, status_label) = match result.diff_type {
                DiffType::NewSeats => ("🟢", "Nouvelles places"),
                DiffType::RemovedSeats => ("🔴", "Places retirées"),
            };

            let resale = match &encounter.resale_link {
                Some(link) => format!("\n🔗 <a href=\"{}\">Accéder à la revente</a>", link),
                None => String::new(),
            };

            let encounter_header = format!(
                "{}\n\n━━━━━━━━━━━━━━━━\n\n🆚 <b>{}</b>\n📅 <i>{}</i>\n\n{} <b>{} :</b>{}",
                header, encounter.title, encounter.date, status_icon, status_label, resale,
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