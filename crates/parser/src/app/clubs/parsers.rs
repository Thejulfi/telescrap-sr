use crate::core::encounter::Encounter;
use crate::core::seat::Seat;

pub trait ParseMatch {
    fn parse_match(&self, html: &str) -> Vec<Encounter>;
}

pub trait ParseSeat {
    fn parse_seat(&self, html: &str, encounter: Encounter) -> Vec<Seat>;
}
