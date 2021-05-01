use std::{env, str::FromStr, collections::HashMap};
use log::{self, error, LevelFilter};
use simple_logger::SimpleLogger;
use lambda::{handler_fn, Context};
use aws_lambda_events::event::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use rusoto_core::Region;
use common::subscriptions_repo::{get_subscription_by_auth_token, store_subscription};

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

    let auth_token = match event.query_string_parameters.get("auth_token") {
        Some(auth_token) => auth_token,
        None => {
            return Ok(create_response(400, "Missing authentication token".to_owned()));
        }
    };

    let mut subscription = match get_subscription_by_auth_token(&subscriptions_table, &region, auth_token).await {
        Ok(optional_subscription) => match optional_subscription {
            Some(subscription) => subscription,
            None => return Ok(create_response(404, "Subscription not found".to_owned()))
        },
        Err(error) => {
            error!("Failed to read from database: {}", error);
            return Ok(create_response(500, "Internal error when verifying subscription".to_owned()))
        }
    };

    if subscription.is_authenticated {
        return Ok(create_response(400, "Subscription already verified".to_owned()))
    }

    subscription.verify();

    match store_subscription(&subscriptions_table, &region, &subscription).await {
        Ok(()) => Ok(create_response(200, "Successfully verified subscription".to_owned())),
        Err(error) => {
            error!("Failed to write to database: {}", error);
            Ok(create_response(500, "Failed to write to database".to_owned()))
        }
    }
}

fn create_response(status_code: i64, body: String) -> ApiGatewayV2httpResponse {
    ApiGatewayV2httpResponse {
        status_code,
        headers: HashMap::new(),
        multi_value_headers: HashMap::new(),
        body: Some(body),
        is_base64_encoded: None,
        cookies: Vec::new()
    }
}
