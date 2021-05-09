use aws_lambda_events::event::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use common::send_email::send_email;
use common::subscription::Subscription;
use common::subscriptions_repo::{get_subscription, store_subscription};
use common::stops_repo::get_single_stop;
use lambda::{handler_fn, Context};
use log::{self, error, LevelFilter};
use rusoto_core::Region;
use simple_logger::SimpleLogger;
use std::{collections::HashMap, env, str::FromStr};

mod add_subscription_request;
mod parser;
mod verification_email;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _log = SimpleLogger::new().with_level(LevelFilter::Info).init();
    let handler = handler_fn(handle_request);
    lambda::run(handler).await?;
    Ok(())
}

async fn handle_request(
    event: ApiGatewayV2httpRequest,
    _: Context,
) -> Result<ApiGatewayV2httpResponse, Error> {
    let subscriptions_table = env::var("SUBSCRIPTIONS_TABLE").unwrap();
    let events_table = env::var("EVENTS_TABLE").unwrap();
    let location_index = env::var("LOCATION_INDEX").unwrap();
    let verify_url = env::var("VERIFY_URL").unwrap();
    let api_key = env::var("SENDGRID_API_KEY").unwrap();
    let email_domain = env::var("EMAIL_DOMAIN").unwrap();
    let region = env::var("AWS_REGION").unwrap();
    let region = Region::from_str(&region).unwrap();

    let request = match parser::parse_request(event) {
        Ok(request)  => request,
        Err(e) => return Ok(create_response(e.status_code, e.message))
    };

    match get_subscription(&subscriptions_table, &region, &request.email, &request.location_id,).await {
        Ok(optional_subscription) => if let Some(subscription) = optional_subscription {
            if subscription.is_authenticated {
                return Ok(create_response(
                    400,
                    "Subscription already exist for this e-mail address and location".to_owned(),
                ));
            }
        },
        Err(error) => {
            error!("Failed to read from database: {}", error);
            return Ok(create_response(
                500,
                "Failed to read from database".to_owned(),
            ));
        }
    }

    let stop = match get_single_stop(&events_table, &region, &location_index, &request.location_id).await {
        Ok(optional_stop) => match optional_stop {
            Some(stop) => stop,
            None => {
                return Ok(create_response(400, format!("Location does not exist: {}", request.location_id)));
            }
        },
        Err(error) => {
            error!("Failed to read from database: {}", error);
            return Ok(create_response(500, "Failed to read from database".to_owned()));
        }
    };

    let subscription = Subscription::new(request.email, request.location_id);
    match store_subscription(&subscriptions_table, &region, &subscription).await {
        Ok(()) => (),
        Err(error) => {
            error!("Failed to write to database: {}", error);
            return Ok(create_response(
                500,
                "Failed to write to database".to_owned(),
            ));
        }
    };

    let email_request = verification_email::create_request(&subscription, &stop, &email_domain, &verify_url); 
    match send_email(&api_key, email_request).await {
        Ok(_response) => Ok(create_response(200, "Successfully created subscription".to_owned())),
        Err(error) => {
            error!("Failed to send verification email: {}", error);
            Ok(create_response(500, "Failed to send verification email".to_owned()))
        }
    }
}

fn create_response(status_code: i64, body: String) -> ApiGatewayV2httpResponse {
    ApiGatewayV2httpResponse {
        status_code,
        headers: HashMap::new(),
        multi_value_headers: HashMap::new(),
        body: Some(body),
        is_base64_encoded: Some(false),
        cookies: Vec::new()
    }
}
