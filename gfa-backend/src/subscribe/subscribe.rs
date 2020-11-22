use std::{error, collections::HashMap};
use rusoto_core::{Region};
use rusoto_sns::{Sns, SnsClient, SubscribeInput};
use crate::subscription::Subscription;

pub async fn subscribe(subscription: &Subscription, topic_arn: String, region: Region) -> Result<(), Box<dyn error::Error>> {
    let client = SnsClient::new(region);

    let mut message_attributes: HashMap<String, String> = HashMap::new();
    message_attributes.insert("FilterPolicy".to_string(), format!("{{\"location_id\": [\"{}\"]}}", subscription.location_id));
    let input = SubscribeInput{
        protocol: "email".to_string(),
        endpoint: Some(subscription.email.clone()),
        attributes: Some(message_attributes),
        topic_arn: topic_arn,
        ..Default::default()
    };

    match client.subscribe(input).await {
        Ok(output) => output,
        Err(e) => return Err(Box::new(e)),
    };
    Ok(())
}
