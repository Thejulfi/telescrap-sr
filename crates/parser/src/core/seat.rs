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
    pub seat_info: Option<String>,
    pub price: Option<String>,
    pub actions: SeatAction,
}