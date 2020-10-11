use std::fmt;
use std::error::Error;
use chrono::{DateTime, Utc};

#[derive(fmt::Debug)]
pub struct PickUpEvent {
    street: String,
    district: String,
    description: Option<String>,
    time_start: String,
    time_end: String,
}

impl fmt::Display for PickUpEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {} ({}): {} to {}\n", self.district, self.street, self.description.as_ref().unwrap_or(&"-".to_string()), self.time_start, self.time_end)
    }
}

impl PickUpEvent {
    pub fn new(street: String, district: String, description: Option<String>, time_start: String, time_end: String) -> Result<Self, Box<Error>> {
        let time_start = DateTime::parse_from_rfc3339(&time_start)?
            .with_timezone(&Utc);
        let time_end = DateTime::parse_from_rfc3339(&time_end)?
            .with_timezone(&Utc);
        Ok(PickUpEvent{
            street,
            district,
            description,
            time_start: time_start.to_rfc3339(),
            time_end: time_end.to_rfc3339()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_convert_to_utc() {
        let event = PickUpEvent::new("Sunnerviksgatan 38".to_string(), "Västra Hisingen".to_string(), Some("jättestensskolan".to_string()), "2020-09-23T18:00:00+02:00".to_string(), "2020-09-23T18:45:00+02:00".to_string()).unwrap();
        assert_eq!("Västra Hisingen - Sunnerviksgatan 38 (jättestensskolan): 2020-09-23T16:00:00+00:00 to 2020-09-23T16:45:00+00:00\n".to_string(), event.to_string());
    }

    #[test]
    fn should_fail_on_invalid_time() {
        let event = PickUpEvent::new("Sunnerviksgatan 38".to_string(), "Västra Hisingen".to_string(), Some("jättestensskolan".to_string()), "2020-09-23TKASS_TID".to_string(), "KASST_DATUMT18:45:00+02:00".to_string());
        assert_eq!(true, event.is_err());
    }
}