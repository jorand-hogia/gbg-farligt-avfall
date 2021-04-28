use std::error::Error;
use chrono::{DateTime};
use chrono_tz::Europe::Stockholm;
use common::pickup_event::PickUpEvent;

pub fn format_email_message(event: &PickUpEvent) -> Option<String> {
    let email_content = include_str!("notify_email.html");
    let email_content = email_content.replace("#STREET#", &event.street);
    let description = event.description.as_ref().map_or("".to_owned(), |description| format!("({})", description));
    let email_content = email_content.replace("#DESCRIPTION#", &description);

    let date = match rfc3339_string_to_date(&event.time_start) {
        Ok(date) => date,
        Err(_e) => return None,
    };
    let email_content = email_content.replace("#DATE#", &date);

    let start_time = match rfc3339_string_to_local_time(&event.time_start) {
        Ok(start_time) => start_time,
        Err(_e) => return None,
    };
    let email_content = email_content.replace("#START#", &start_time);

    let end_time = match rfc3339_string_to_local_time(&event.time_end) {
        Ok(end_time) => end_time,
        Err(_e) => return None,
    };
    let email_content = email_content.replace("#END#", &end_time);

    Some(email_content)
}

fn rfc3339_string_to_date(rfc_string: &str) -> Result<String, Box<dyn Error>> {
    let date_time = DateTime::parse_from_rfc3339(rfc_string)?
        .with_timezone(&Stockholm);
    Ok(date_time.format("%Y-%m-%d").to_string())
}

fn rfc3339_string_to_local_time(rfc_string: &str) -> Result<String, Box<dyn Error>> {
    let date_time = DateTime::parse_from_rfc3339(rfc_string)?
        .with_timezone(&Stockholm);
    Ok(date_time.format("%H:%M").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_convert_rfc3339_string_correctly_daylight_saving() {
        let result = rfc3339_string_to_local_time("2020-06-06T06:00:00+00:00");
        assert_eq!("08:00".to_string(), result.unwrap());
    }

    #[test]
    fn should_convert_rfc3339_string_correctly() {
        let result = rfc3339_string_to_local_time("2020-11-18T06:00:00+00:00");
        assert_eq!("07:00".to_string(), result.unwrap());
    }
}
