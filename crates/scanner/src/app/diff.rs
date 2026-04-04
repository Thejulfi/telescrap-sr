use parser::core::encounter::Encounter;

#[derive(Debug, Clone)]
pub enum DiffType {
    NewSeats,
    RemovedSeats,
}

#[derive(Debug, Clone)]
pub struct DiffResult {
    pub diff_type: DiffType,
    pub encounter_diff_only: Encounter,
}

pub fn diff(previous: &[Encounter], current: &[Encounter]) -> Vec<DiffResult> {
    let mut results = Vec::new();

    for encounter in current {
        let prev = previous.iter().find(|e| e.title == encounter.title);
        let current_seats = encounter.seats.as_deref().unwrap_or(&[]);

        match prev {
            None => {
                if !current_seats.is_empty() {
                    results.push(DiffResult {
                        diff_type: DiffType::NewSeats,
                        encounter_diff_only: encounter.clone(),
                    });
                }
            }
            Some(p) => {
                let prev_seats = p.seats.as_deref().unwrap_or(&[]);

                let added: Vec<_> = current_seats
                    .iter()
                    .filter(|s| !prev_seats.iter().any(|ps| ps.actions.pack_id == s.actions.pack_id))
                    .cloned()
                    .collect();

                let removed: Vec<_> = prev_seats
                    .iter()
                    .filter(|s| !current_seats.iter().any(|cs| cs.actions.pack_id == s.actions.pack_id))
                    .cloned()
                    .collect();

                if !added.is_empty() {
                    let mut enc = encounter.clone();
                    enc.seats = Some(added);
                    results.push(DiffResult { diff_type: DiffType::NewSeats, encounter_diff_only: enc });
                }

                if !removed.is_empty() {
                    let mut enc = encounter.clone();
                    enc.seats = Some(removed);
                    results.push(DiffResult { diff_type: DiffType::RemovedSeats, encounter_diff_only: enc });
                }
            }
        }
    }

    results
}
