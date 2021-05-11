use std::{error, fmt, collections::HashMap};
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, GetItemInput, PutItemInput, QueryInput, DeleteItemInput, AttributeValue};
use rusoto_core::{Region};
use log::{self, warn};
use crate::subscription::Subscription;
use crate::dynamodb_util::MalformedDynamoDbResponse;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
struct MalformedSubscription {
    email: Option<String>,
    token: Option<String>
}
impl fmt::Display for MalformedSubscription {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut error_message = "Malformed subscription in database.".to_owned();
    if self.email.is_some() {
        error_message.push_str(&format!("Email: {}.", self.email.as_ref().unwrap()));
    }
    if self.token.is_some() {
        error_message.push_str(&format!("Token: {}.", self.token.as_ref().unwrap()));
    }
    write!(f, "{}", error_message)
  } 
}
impl error::Error for MalformedSubscription {}

#[derive(Debug)]
struct TokenCollision;
impl fmt::Display for TokenCollision {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "More than two subscriptions were found with the same token")
    }
}
impl error::Error for TokenCollision {}

pub async fn store_subscription(table: &str, region: &Region, subscription: &Subscription) -> Result<(), Error> {
    let client = DynamoDbClient::new(region.clone());

    let mut attributes: HashMap<String, AttributeValue> = HashMap::new(); 
    attributes.insert("email".to_owned(), AttributeValue{
        s: Some(subscription.email.clone()),
        ..Default::default()
    });
    attributes.insert("location_id".to_owned(), AttributeValue{
        s: Some(subscription.location_id.clone()),
        ..Default::default()
    });
    if subscription.auth_token.is_some() {
        attributes.insert("auth_token".to_owned(), AttributeValue{
            s: Some(subscription.auth_token.as_ref().unwrap().clone()),
            ..Default::default()
        });
    }
    if subscription.unsubscribe_token.is_some() {
        attributes.insert("unsubscribe_token".to_owned(), AttributeValue{
            s: Some(subscription.unsubscribe_token.as_ref().unwrap().clone()),
            ..Default::default()
        });
    }
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
        table_name: table.to_owned(),
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

pub async fn get_subscription(table: &str, region: &Region, email: &str, location_id: &str) -> Result<Option<Subscription>, Error> {
    let client = DynamoDbClient::new(region.clone());

    let mut attributes: HashMap<String, AttributeValue> = HashMap::new();
    attributes.insert("email".to_owned(), AttributeValue{
        s: Some(email.to_owned()),
        ..Default::default()
    });
    attributes.insert("location_id".to_owned(), AttributeValue{
        s: Some(location_id.to_owned()),
        ..Default::default()
    });

    match client.get_item(GetItemInput{
        table_name: table.to_owned(),
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
            match item_to_subscription(&item) {
                Some(subscription) => Ok(Some(subscription)),
                None => {
                    Err(Box::new(MalformedSubscription{
                        email: Some(email.to_owned()),
                        token: None,
                    }))
                }
            }
        },
        Err(err) => {
            Err(Box::new(err))
        }
    }
}

pub async fn get_subscription_by_unsubscribe_token(table: &str, region: &Region, unsubscribe_token: &str) -> Result<Option<Subscription>, Error> {
    get_subscription_by_token(table, region, "byUnsubscribeToken", "unsubscribe_token", unsubscribe_token).await
}

pub async fn get_subscription_by_auth_token(table: &str, region: &Region, auth_token: &str) -> Result<Option<Subscription>, Error> {
    get_subscription_by_token(table, region, "byAuthToken", "auth_token", auth_token).await
}

async fn get_subscription_by_token(table: &str, region: &Region, index_name: &str, property_name: &str, value: &str) -> Result<Option<Subscription>, Error> {
    let client = DynamoDbClient::new(region.clone());
    let mut attribute_values = HashMap::new();
    attribute_values.insert(format!(":{}", property_name), AttributeValue{
        s: Some(value.to_owned()),
        ..Default::default()
    });
    match client.query(QueryInput{
        index_name: Some(index_name.to_owned()),
        table_name: table.to_owned(),
        expression_attribute_values: Some(attribute_values),
        key_condition_expression: Some(format!("{} = :{}", property_name, property_name)),
        ..Default::default()
    }).await {
        Ok(response) => {
            let items = match response.items {
                Some(items) => items,
                None => return Err(Box::new(MalformedDynamoDbResponse))
            };
            if items.is_empty() {
                return Ok(None)
            }
            if items.len() > 1 {
                return Err(Box::new(TokenCollision{}))
            }
            let item = items.first().unwrap();
            match item_to_subscription(item) {
                Some(subscription) => Ok(Some(subscription)),
                None => {
                    Err(Box::new(MalformedSubscription{
                        email: None,
                        token: Some(value.to_owned())
                    }))
                }
            }

        },
        Err(error) => {
            Err(Box::new(error))
        }
    }
}

pub async fn get_authenticated_subscriptions(table: &str, region: &Region, location_id: &str) -> Result<Vec<Subscription>, Error> {
    let client = DynamoDbClient::new(region.clone());
    let mut attribute_values = HashMap::new();
    attribute_values.insert(":locationId".to_owned(), AttributeValue{
        s: Some(location_id.to_owned()),
        ..Default::default()
    });
    match client.query(QueryInput{
        index_name: Some("byLocationId".to_owned()),
        table_name: table.to_owned(),
        expression_attribute_values: Some(attribute_values),
        key_condition_expression: Some("location_id = :locationId".to_owned()),
        ..Default::default()
    }).await {
        Ok(response) => {
           let items = match response.items {
               Some(items) => items,
               None => return Err(Box::new(MalformedDynamoDbResponse))
           };
           Ok(items.iter()
                .filter_map(|item| match item_to_subscription(item) {
                    Some(subscription) => Some(subscription),
                    None => {
                        warn!("Found malformed subscription: {:?}", item);
                        None
                    }
                })
                .filter(|subscription| subscription.is_authenticated)
                .collect())
        },
        Err(error) => {
            Err(Box::new(error))
        }
    }
}

pub async fn remove_subscription(table: &str, region: &Region, subscription: &Subscription) -> Result<(), Error> {
    let client = DynamoDbClient::new(region.clone());
    match client.delete_item(DeleteItemInput{
        table_name: table.to_owned(),
        key: [
            ("email".to_owned(), AttributeValue{s: Some(subscription.email.to_owned()), ..Default::default()}),
            ("location_id".to_owned(), AttributeValue{s: Some(subscription.location_id.to_owned()), ..Default::default()})
        ].iter().cloned().collect(),
        ..Default::default()
    }).await {
        Ok(_output) => Ok(()),
        Err(error) => Err(Box::new(error))
    }
}

fn item_to_subscription(item: &HashMap<String, AttributeValue>) -> Option<Subscription> {
    let email = item.get("email")?.s.as_ref()?;
    let location_id = item.get("location_id")?.s.as_ref()?;
    let is_authenticated = item.get("is_authenticated")?.bool.as_ref()?;
    let auth_token = match item.get("auth_token") {
        None => None,
        Some(auth_token) => Some(auth_token.s.as_ref()?.clone())
    };
    let unsubscribe_token = match item.get("unsubscribe_token") {
        None => None,
        Some(unsubscribe_token) => Some(unsubscribe_token.s.as_ref()?.clone())
    };
    let ttl = match item.get("ttl") {
        None => None,
        Some(ttl) => Some(ttl.n.as_ref()?.parse::<i64>().ok()?)
    };
    Some(Subscription{
        email: email.clone(),
        location_id: location_id.clone(),
        auth_token,
        unsubscribe_token,
        is_authenticated: *is_authenticated,
        ttl,
    })
}
