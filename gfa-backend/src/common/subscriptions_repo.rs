use std::{error, fmt, collections::HashMap};
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, GetItemInput, PutItemInput, AttributeValue};
use rusoto_core::{Region};
use crate::subscription::Subscription;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
struct MalformedSubscription {
    email: String,
    location_id: String
}
impl fmt::Display for MalformedSubscription {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Malformed subscription in database for: {} - {}", self.email, self.location_id)
  } 
}
impl error::Error for MalformedSubscription {}

pub async fn store_subscription(table: &String, region: &Region, subscription: Subscription) -> Result<(), Error> {
    let client = DynamoDbClient::new(region.clone());

    let mut attributes: HashMap<String, AttributeValue> = HashMap::new(); 
    attributes.insert("email".to_owned(), AttributeValue{
        s: Some(subscription.email),
        ..Default::default()
    });
    attributes.insert("location_id".to_owned(), AttributeValue{
        s: Some(subscription.location_id),
        ..Default::default()
    });
    attributes.insert("auth_token".to_owned(), AttributeValue{
        s: Some(subscription.auth_token),
        ..Default::default()
    });
    attributes.insert("is_authenticated".to_owned(), AttributeValue{
        bool: Some(subscription.is_authenticated),
        ..Default::default()
    });
    if subscription.ttl.is_some() {
        attributes.insert("ttl".to_owned(), AttributeValue{
            n: Some(subscription.ttl.unwrap().to_string()),
            ..Default::default()
        });
    }

    match client.put_item(PutItemInput{
        item: attributes,
        table_name: table.clone(),
        ..Default::default()
    }).await {
        Ok(_response) => {
            Ok(())
        },
        Err(err) => {
            Err(Box::new(err))
        }
    }
}

pub async fn get_subscription(table: &String, region: &Region, email: &String, location_id: &String) -> Result<Option<Subscription>, Error>{
    let client = DynamoDbClient::new(region.clone());

    let mut attributes: HashMap<String, AttributeValue> = HashMap::new();
    attributes.insert("email".to_owned(), AttributeValue{
        s: Some(email.clone()),
        ..Default::default()
    });
    attributes.insert("location_id".to_owned(), AttributeValue{
        s: Some(location_id.clone()),
        ..Default::default()
    });

    match client.get_item(GetItemInput{
        table_name: table.clone(),
        key: attributes,
        ..Default::default()
    }).await {
        Ok(response) => {
            let item = match response.item {
                Some(item) => item,
                None => {
                    return Ok(None)
                }
            };
            match item_to_subscription(item) {
                Some(subscription) => Ok(Some(subscription)),
                None => {
                    Err(Box::new(MalformedSubscription{
                        email: email.clone(),
                        location_id: location_id.clone()
                    }))
                }
            }
        },
        Err(err) => {
            Err(Box::new(err))
        }
    }
}

fn item_to_subscription(item: HashMap<String, AttributeValue>) -> Option<Subscription> {
    let email = item.get("email")?.s.as_ref()?;
    let location_id = item.get("location_id")?.s.as_ref()?;
    let auth_token = item.get("auth_token")?.s.as_ref()?;
    let is_authenticated = item.get("is_authenticated")?.bool.as_ref()?;
    let ttl = match item.get("ttl") {
        None => None,
        Some(ttl) => Some(ttl.n.as_ref()?.parse::<i64>().ok()?)
    };
    Some(Subscription{
        email: email.clone(),
        location_id: location_id.clone(),
        auth_token: auth_token.clone(),
        is_authenticated: is_authenticated.clone(),
        ttl,
    })
}
