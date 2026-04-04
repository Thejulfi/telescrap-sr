
use crate::core::seat::SeatAction;

pub trait FetchHtml {
    fn get_html(&self, url: &str) -> Result<String, String>;
    fn add_to_cart(&self, action: &SeatAction) -> Result<(), String>;
}

pub fn extract_html(url: &str, client: &impl FetchHtml) -> Result<String, String> {
    client.get_html(url)
}