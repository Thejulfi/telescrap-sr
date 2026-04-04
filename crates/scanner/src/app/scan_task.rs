
use tokio::time::{interval, Duration};
use crate::core::scan::{ScanConfig, ScanResult, ScanMode};
use crate::controller::notify::Notify;
use crate::app::diff::{diff, DiffResult, DiffType};
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
            let has_filter = self.config.filter.is_some();
            let is_aggressive = self.config.mode == ScanMode::AggressiveScan;
            let scan_result = tokio::task::spawn_blocking(move || {
                let encounters = match_manager::get_seats_from_matches(Some(club), Some(self.config.nature));
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

            if changed.is_empty() {
                if self.previous.is_some() {
                    println!("Aucun changement depuis le dernier scan.");
                } else {
                    println!("Premier scan effectué, résultats enregistrés.");
                }
            } else {
                self.notify_parsed_info(&scan_result, &changed);
            }

            self.previous = Some(scan_result);
        }
    }

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
                    .filter(|s| s.price.as_deref().and_then(|p| p.trim_end_matches('€').parse::<f64>().ok()).map_or(false, |p| p <= threshold))
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
                    let ac = a.seat_info.as_ref().map(|i| (i.composition.access.as_str(), i.composition.row.as_str(), i.composition.seat_number));
                    let bc = b.seat_info.as_ref().map(|i| (i.composition.access.as_str(), i.composition.row.as_str(), i.composition.seat_number));
                    ac.partial_cmp(&bc).unwrap_or(std::cmp::Ordering::Equal)
                });

                // Find indices that belong to a run of >= required consecutive seats
                let mut in_group = vec![false; sorted.len()];
                let mut run_start = 0;
                for i in 1..=sorted.len() {
                    let consecutive = i < sorted.len() && sorted[i].seat_info.as_ref().zip(sorted[i - 1].seat_info.as_ref()).map_or(false, |(cur, prev)| {
                        cur.composition.access == prev.composition.access
                            && cur.composition.row == prev.composition.row
                            && cur.composition.seat_number == prev.composition.seat_number + 1
                    });
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

    fn notify_parsed_info(&self, scan_result: &ScanResult, changed: &[DiffResult]) {
        println!("Scanned at: {:?}", scan_result.scanned_at);

        let scanned_at_str = self.get_scanner_time_str(scan_result);

        let mut message = format!(
            "🏉 <b>{}</b>\n<i>{}</i>\n",
            self.config.club.name,
            scanned_at_str,
        );

        for result in changed {
            let encounter = &result.encounter_diff_only;
            let (status_icon, status_label) = match result.diff_type {
                DiffType::NewSeats => ("🟢", "Nouveaux sièges"),
                DiffType::RemovedSeats => ("🔴", "Sièges retirés"),
            };

            let seat_list = match &encounter.seats {
                Some(seats) if !seats.is_empty() => seats
                    .iter()
                    .map(|s| format!(
                        "  • {} — <code>{}€ </code>",
                        s.seat_info.as_ref().map(|info| &info.full_name).map_or("?", |v| v),
                        s.price.as_deref().unwrap_or("prix inconnu"),
                    ))
                    .collect::<Vec<_>>()
                    .join("\n"),
                _ => "  <i>Aucun siège disponible</i>".to_string(),
            };

            let resale = match &encounter.resale_link {
                Some(link) => format!("\n\n🔗 <a href=\"{}\">Accéder à la revente</a>", link),
                None => String::new(),
            };

            message.push_str(&format!(
                "\n\n━━━━━━━━━━━━━━━━\n\n🆚 <b>{}</b>\n📅 <i>{}</i>\n\n{} <b>{} :</b>\n\n{}{}",
                encounter.title, encounter.date, status_icon, status_label, seat_list, resale,
            ));
        }

        self.notifier.send(&message);
    }

    fn get_scanner_time_str(&self, scan_result: &ScanResult) -> String {
        let secs = scan_result.scanned_at
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let (h, m, s) = (secs % 86400 / 3600, secs % 3600 / 60, secs % 60);
        let days_since_epoch = secs / 86400;
        // Simple Gregorian date from days since 1970-01-01
        let (mut y, mut doy) = (1970u32, days_since_epoch as u32);
        loop {
            let dy = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 366 } else { 365 };
            if doy < dy { break; }
            doy -= dy;
            y += 1;
        }
        let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
        let months = [31u32, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let (mut mo, mut d) = (1u32, doy + 1);
        for days_in_month in months {
            if d <= days_in_month { break; }
            d -= days_in_month;
            mo += 1;
        }
        format!("{:02}/{:02}/{} à {:02}:{:02}:{:02}", d, mo, y, h, m, s)
    }
}