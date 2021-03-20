use std::{error, fmt, collections::HashMap};
use rusoto_core::{Region, RusotoError};
use rusoto_sns::{SnsClient, Sns, PublishInput, MessageAttributeValue, PublishError};
use chrono::{DateTime};
use chrono_tz::Europe::Stockholm;
use log::{self, info};
use common::pickup_event::PickUpEvent;

type Error = Box<dyn error::Error + Send + Sync + 'static>;

#[derive(fmt::Debug)]
pub struct PublishEventsError {
    pub errors: Vec<RusotoError<PublishError>>,
}
impl fmt::Display for PublishEventsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for error in &self.errors {
            write!(f, "{}\n", error)?;
        }
        write!(f, "Total db errors while writing events: {}", self.errors.len())
    }
}
impl error::Error for PublishEventsError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None 
    }
}

pub async fn publish_events(events: Vec<PickUpEvent>, topic_arn: String, region: Region) -> Result<(), Error> {
    let sns_client = SnsClient::new(region);
    let mut errors: Vec<RusotoError<PublishError>> = Vec::new();
    for event in events {
        let publish_input = create_sns_publish_input(&event, topic_arn.clone())?;
        match sns_client.publish(publish_input).await {
            Err(e) => {
                errors.push(e);
            },
            _ => {
                info!("Successfully published notification for {}", event.location_id);
            }
        };
    }
    if !errors.is_empty() {
        return Err(Box::new(PublishEventsError{
            errors,
        }));
    }
    Ok(())
} 

fn create_sns_publish_input(event: &PickUpEvent, topic_arn: String) -> Result<PublishInput, Error> {
    let mut message_attributes: HashMap<String, MessageAttributeValue> = HashMap::new();
    message_attributes.insert("location_id".to_string(), MessageAttributeValue{
        data_type: "String".to_string(),
        string_value: Some(String::from(&event.location_id)),
        ..Default::default()
    });
    Ok(PublishInput{
        subject: Some(format!("Farligt avfall-bilen")),
        message: format!("Farligt avfall-bilen will arrive to {} today at {}-{}",
            event.street,
            rfc3339_string_to_local_time(String::from(&event.time_start))?,
            rfc3339_string_to_local_time(String::from(&event.time_end))?
        ),
        message_attributes: Some(message_attributes),
        topic_arn: Some(topic_arn.clone()),
        ..Default::default()
    })
}

fn rfc3339_string_to_local_time(rfc_string: String) -> Result<String, Error>{
    let date_time = DateTime::parse_from_rfc3339(&rfc_string)?
        .with_timezone(&Stockholm);
    Ok(date_time.format("%H:%M").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_event() -> PickUpEvent {
        PickUpEvent::new("Hittepåvägen 14".to_string(), "Hisingen".to_string(), None, "2020-06-06T06:00:00+00:00".to_string(), "2020-06-06T07:00:00+00:00".to_string()).unwrap()
    }

    #[test]
    fn should_include_correct_location_id() {
        let event = create_test_event();
        let publish_input = create_sns_publish_input(&event, "some-topic".to_string());
        assert_eq!("hisingen_hittepåvägen14".to_string(), publish_input.unwrap().message_attributes.unwrap().get("location_id").unwrap().string_value.as_ref().unwrap().clone());
    }

    #[test]
    fn should_include_correct_message() {
        let event = create_test_event();
        let publish_input = create_sns_publish_input(&event, "some-topic".to_string());
        assert_eq!("Farligt avfall-bilen will arrive to Hittepåvägen 14 today at 08:00-09:00".to_string(), publish_input.unwrap().message);
    }

    #[test]
    fn should_convert_rfc3339_string_correctly_daylight_saving() {
        let result = rfc3339_string_to_local_time("2020-06-06T06:00:00+00:00".to_string());
        assert_eq!("08:00".to_string(), result.unwrap());
    }

    #[test]
    fn should_convert_rfc3339_string_correctly() {
        let result = rfc3339_string_to_local_time("2020-11-18T06:00:00+00:00".to_string());
        assert_eq!("07:00".to_string(), result.unwrap());
    }
}
