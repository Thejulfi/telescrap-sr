use parser::core::encounter::Encounter;

pub fn diff(previous: &[Encounter], current: &[Encounter]) -> Vec<Encounter> {
    current
        .iter()
        .filter(|encounter| {
            let prev = previous.iter().find(|e| e.title == encounter.title);
            match (prev, &encounter.seats) {
                (None, Some(seats)) => !seats.is_empty(),
                (Some(p), Some(seats)) => {
                    let prev_ids: Vec<&str> = p
                        .seats
                        .as_ref()
                        .map_or(vec![], |s| s.iter().map(|s| s.actions.pack_id.as_str()).collect());
                    let curr_ids: Vec<&str> =
                        seats.iter().map(|s| s.actions.pack_id.as_str()).collect();
                    prev_ids != curr_ids
                }
                _ => false,
            }
        })
        .cloned()
        .collect()
}
