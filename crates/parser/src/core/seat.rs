#[derive(Debug, Clone)]
pub struct SeatComposition {
    pub access: String,
    pub row: String,
    pub seat_number: u64,
}

#[derive(Debug, Clone)]
pub struct SeatInfo {
    pub full_name: String,
    pub composition: SeatComposition,
}

#[derive(Debug, Clone)]
pub struct SeatAction {
    pub pack_id: String,
    pub ticket_id: String,
    pub ajax_url: String,
    pub price_min: String,
    pub price_max: String,
    pub libraries: String,
    pub form_build_id: String,
    pub form_token: String,
}

#[derive(Debug, Clone)]
pub struct Seat {
    pub seat_info: Option<SeatInfo>,
    pub price: Option<String>,
    pub actions: SeatAction,
}