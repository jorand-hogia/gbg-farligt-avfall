use chrono::{Date, DateTime, Utc, Duration};
use common::pickup_event::PickUpEvent;

pub fn filter(events: Vec<PickUpEvent>, today: Date<Utc>) -> Vec<PickUpEvent> {
    events.into_iter()
        .filter(|event| {
            let event_date = DateTime::parse_from_rfc3339(&event.time_start).unwrap()
                .with_timezone(&Utc)
                .date();
            let too_far_into_future = today + Duration::weeks(24);
            if event_date > too_far_into_future {
                return false;
            }
            true
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime};

    #[test]
    fn shoult_filter_items_too_far_into_future() {
        // Events are added to the page twice a year
        // When scraping events, I assume that the events are for the current year, but I can't assume that the page has been updated with a new schedule on 1st of January
        // So, if I here get an event very long into the future, it just probably hasn't been updated..
        let pseudo_today = DateTime::parse_from_rfc3339("2021-01-01T00:00:00+00:00").unwrap()
            .with_timezone(&Utc)
            .date();
        let events: Vec<PickUpEvent> = vec![
            PickUpEvent::new("some-street".to_owned(), "some-district".to_owned(), None, "2021-08-14T16:00:00+02:00".to_owned(), "2021-08-14T17:00:00+02:00".to_owned()).unwrap(),
            PickUpEvent::new("some-other-street".to_owned(), "some-other-district".to_owned(), None, "2021-05-15T16:00:00+02:00".to_owned(), "2020-05-15T17:00:00+02:00".to_owned()).unwrap()
        ];
        let result = filter(events, pseudo_today);
        assert_eq!(1, result.len());
        assert_eq!("some-other-street".to_owned(), result[0].street);
    }
}
