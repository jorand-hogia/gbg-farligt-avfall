use std::{collections::HashMap, env, str::FromStr};
use simple_logger::SimpleLogger;
use log::{self, error, LevelFilter};
use lambda::{handler_fn, Context};
use aws_lambda_events::event::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use rusoto_core::Region;
use common::subscriptions_repo::{get_subscription_by_unsubscribe_token, remove_subscription};

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
    let region = env::var("AWS_REGION").unwrap();
    let region = Region::from_str(&region).unwrap();

    let unsubscribe_token = match event.query_string_parameters.get("unsubscribe_token") {
        Some(unsubscribe_token) => unsubscribe_token,
        None => {
            return Ok(create_response(400, "Missing unsubscribe token".to_owned()));
        }
    };
    let email = match event.query_string_parameters.get("email") {
        Some(email) => email,
        None => {
            return Ok(create_response(400, "Missing email".to_owned()));
        }
    };

    let subscription_from_db = match get_subscription_by_unsubscribe_token(&subscriptions_table, &region, unsubscribe_token).await {
        Ok(optional_subscription) => match optional_subscription {
            Some(subscription) => subscription,
            None => return Ok(create_response(404, "Subscription not found".to_owned()))
        },
        Err(error) => {
            error!("Failed to read from database: {}", error);
            return Ok(create_response(500, "Internal error when removing subscription".to_owned()))
        }
    };

    if subscription_from_db.email != *email {
        return Ok(create_response(400, "Bad unsubscribe token".to_owned()));
    }

    match remove_subscription(&subscriptions_table, &region, &subscription_from_db).await {
        Ok(()) => Ok(create_response(200, "Successfully removed subscription".to_owned())),
        Err(error) => {
            error!("Failed to write to database: {}", error);
            Ok(create_response(500, "Internal error when removing subscription".to_owned()))
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
