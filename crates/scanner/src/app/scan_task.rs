
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

        let scanned_at_str = self.get_scanner_time_str(scan_result);

        let mut message = format!(
            "🏉 <b>{}</b>\n<i>{}</i>\n",
            self.config.club.name,
            scanned_at_str,
        );

        for encounter in changed {
            let seat_list = match &encounter.seats {
                Some(seats) if !seats.is_empty() => seats
                    .iter()
                    .map(|s| format!(
                        "  • {} — <code>{}€ </code>",
                        s.seat_info.as_deref().unwrap_or("?"),
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
                "\n\n━━━━━━━━━━━━━━━━\n\n🆚 <b>{}</b>\n📅 <i>{}</i>\n\n🎟 <b>Places disponibles :</b>\n\n{}{}",
                encounter.title, encounter.date, seat_list, resale,
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