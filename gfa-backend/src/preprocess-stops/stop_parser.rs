use common::pickup_event::PickUpEvent;
use common::pickup_stop::PickUpStop;

pub fn parse_unique_stops(mut events: Vec<PickUpEvent>) -> Vec<PickUpStop> {
    events.sort_by(|a, b| a.partial_cmp(b).unwrap());
    events.dedup();
    events.into_iter()
        .map(|event| PickUpStop::new(
            event.location_id,
            event.street,
            event.district,
            event.description
        ))
        .collect()
} 

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_give_unique_stops_sorted() {
        let events = vec![
            PickUpEvent::new("aaa_gatan".to_string(), "Hisingen".to_string(), Some("Hjalle!".to_string()), "2020-01-01T16:00:00+02:00".to_string(), "2020-01-01T17:00:00+02:00".to_string()).unwrap(),
            PickUpEvent::new("aaa_gatan".to_string(), "Hisingen".to_string(), Some("Hjalle!".to_string()), "2020-06-06T16:00:00+02:00".to_string(), "2020-06-06T17:00:00+02:00".to_string()).unwrap(),
            PickUpEvent::new("zzz_gatan".to_string(), "Hisingen".to_string(), Some("Wieselgren".to_string()), "2020-06-06T16:00:00+02:00".to_string(), "2020-06-06T17:00:00+02:00".to_string()).unwrap(),
        ];
        let stops = parse_unique_stops(events);
        assert_eq!(2, stops.len());
        assert_eq!(PickUpStop::new("hisingen_aaa_gatan".to_string(), "aaa_gatan".to_string(), "Hisingen".to_string(), Some("Hjalle!".to_string())), *stops.get(0).unwrap());
        assert_eq!(PickUpStop::new("hisingen_zzz_gatan".to_string(), "zzz_gatan".to_string(), "Hisingen".to_string(), Some("Wieselgren".to_string())), *stops.get(1).unwrap());
    }
}
