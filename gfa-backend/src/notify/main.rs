use std::{env, fmt, error, str::FromStr};
use lambda::{handler_fn, Context};
use serde_json::{json, Value};
use simple_logger::{SimpleLogger};
use log::{self, error, info, LevelFilter};
use rusoto_core::Region;
use chrono::{Utc};
use common::events_repo::{get_by_date};
use common::subscriptions_repo::{get_authenticated_subscriptions};
use common::send_email::{send_email, SendEmailRequest, Recipient, From};

mod email_formatter;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
struct MalformedEvent;
impl fmt::Display for MalformedEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Malformed data in event. Could not format notification email.")
    }
}
impl error::Error for MalformedEvent {}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _log = SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init();
    let handler = handler_fn(handle_request);
    lambda::run(handler).await?;
    Ok(())
}

async fn handle_request(_event: Value, _: Context) -> Result<Value, Error> {
    let event_table = env::var("EVENTS_TABLE").unwrap();
    let subscriptions_table = env::var("SUBSCRIPTIONS_TABLE").unwrap();
    let api_key = env::var("SENDGRID_API_KEY").unwrap();
    let email_domain = env::var("EMAIL_DOMAIN").unwrap();
    let unsubscribe_url = env::var("UNSUBSCRIBE_URL").unwrap();
    let region = env::var("AWS_REGION").unwrap();
    let region = Region::from_str(&region).unwrap(); 

    let todays_date = Utc::today().format("%Y-%m-%d").to_string();
    info!("Fetching events for: {}", todays_date);
    let todays_events = get_by_date(event_table, region.clone(), todays_date).await?;
    info!("About to notify for {} events", todays_events.len());

    for event in todays_events {
        let subscriptions = match get_authenticated_subscriptions(&subscriptions_table, &region, &event.location_id).await {
            Ok(subscriptions) => subscriptions, 
            Err(error) => {
                error!("Failed to get subscriptions for: {}", event);
                return Err(error);
            }
        };
        if subscriptions.is_empty() {
            info!("Skipped sending notifications for {}, since there are no subscribers.", event);
            continue;
        }

        let html_content = match email_formatter::format_email_message(&event) {
            Some(content) => content,
            None => {
                error!("Unable to format email for: {}", event);
                return Err(Box::new(MalformedEvent))
            }
        };
        let email_request = SendEmailRequest{
            from: From {
                name: "Göteborg Farligt Avfall Notifications".to_owned(),
                email: format!("noreply-farligtavfall@{}", email_domain),
            },
            subject: format!("Farligt Avfall-bilen to {}", event.street),
            recipients: subscriptions.iter()
                .map(|subscription| Recipient{
                    email: subscription.email.clone(),
                    substitutions: [
                        ("-unsubscribeUrl-".to_owned(), format!("{}?email={}&unsubscribe_token={}",
                            unsubscribe_url,
                            subscription.email.clone(),
                            match subscription.unsubscribe_token.as_ref() {
                                Some(unsubscribe_token) => unsubscribe_token.clone(),
                                None => "MISSING-TOKEN".to_owned() // TODO: Decide on what action to take here
                            }))
                    ]
                    .iter()
                    .cloned()
                    .collect()
                })
                .collect(),
            html_content,
        };
        match send_email(&api_key, email_request).await {
            Ok(_res) => {
                info!("Successfully sent notification email for: {}", event);
            }
            Err(_e) => {
                error!("Unable to send notification email for: {}", event);
            }
        };
    }
    Ok(json!({}))
}
