use std::{fmt, error, cmp::Ordering};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(fmt::Debug, Serialize, Deserialize)]
pub struct PickUpEvent {
    pub location_id: String,
    pub street: String,
    pub district: String,
    pub description: Option<String>,
    pub time_start: String,
    pub time_end: String,
    pub date: String,
}

impl fmt::Display for PickUpEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {} ({}): {} to {}\n", self.district, self.street, self.description.as_ref().unwrap_or(&"-".to_string()), self.time_start, self.time_end)
    }
}

impl Ord for PickUpEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.location_id.cmp(&other.location_id)
    }
}

impl PartialEq for PickUpEvent {
    fn eq(&self, other: &Self) -> bool {
        self.location_id == other.location_id
    }
}
impl Eq for PickUpEvent {}

impl PartialOrd for PickUpEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PickUpEvent {
    pub fn new(street: String, district: String, description: Option<String>, time_start: String, time_end: String) -> Result<Self, Box<dyn error::Error>> {
        let time_start = DateTime::parse_from_rfc3339(&time_start)?
            .with_timezone(&Utc);
        let time_end = DateTime::parse_from_rfc3339(&time_end)?
            .with_timezone(&Utc);
        Ok(PickUpEvent{
            location_id: format!("{}_{}",
                district.to_lowercase().trim().replace(" ", "").replace("/", "-"),
                street.to_lowercase().trim().replace(" ", "").replace("/", "-")
            ),
            street,
            district,
            description,
            time_start: time_start.to_rfc3339(),
            time_end: time_end.to_rfc3339(),
            date: time_start.format("%Y-%m-%d").to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_generate_location_id() {
        let event = PickUpEvent::new("  Doktor Fries torg, Doktor Bondesons Gata ".to_string(), "Centrum".to_string(), Some("jättestensskolan".to_string()), "2020-09-23T18:00:00+02:00".to_string(), "2020-09-23T18:45:00+02:00".to_string()).unwrap();
        assert_eq!("centrum_doktorfriestorg,doktorbondesonsgata", event.location_id);
    }

    #[test]
    fn should_not_include_slash_in_id() {
        let event = PickUpEvent::new("Utmarksgatan/Dysiksgatan".to_string(), "Lundby".to_string(), None, "2020-09-23T18:00:00+02:00".to_string(), "2020-09-23T18:45:00+02:00".to_string()).unwrap();
        assert_eq!("lundby_utmarksgatan-dysiksgatan", event.location_id);
    }

    #[test]
    fn should_generate_date() {
        let event = PickUpEvent::new("Utmarksgatan/Dysiksgatan".to_string(), "Lundby".to_string(), None, "2020-09-23T18:00:00+02:00".to_string(), "2020-09-23T18:45:00+02:00".to_string()).unwrap();
        assert_eq!("2020-09-23".to_string(), event.date);
    }
}
