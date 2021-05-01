use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use common::send_email::{send_email, From, Recipient, SendEmailRequest};
use common::subscription::Subscription;
use common::subscriptions_repo::{get_subscription, store_subscription};
use lambda::{handler_fn, Context};
use log::{self, error, LevelFilter};
use rusoto_core::Region;
use simple_logger::SimpleLogger;
use std::{collections::HashMap, env, str::FromStr};

mod add_subscription_request;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _log = SimpleLogger::new().with_level(LevelFilter::Info).init();
    let handler = handler_fn(handle_request);
    lambda::run(handler).await?;
    Ok(())
}

async fn handle_request(
    event: ApiGatewayProxyRequest,
    _: Context,
) -> Result<ApiGatewayProxyResponse, Error> {
    let subscriptions_table = env::var("SUBSCRIPTIONS_TABLE").unwrap();
    let verify_url = env::var("VERIFY_URL").unwrap();
    let api_key = env::var("SENDGRID_API_KEY").unwrap();
    let email_domain = env::var("EMAIL_DOMAIN").unwrap();
    let region = env::var("AWS_REGION").unwrap();
    let region = Region::from_str(&region).unwrap();

    let body = match event.body {
        Some(body) => body,
        None => {
            return Ok(create_response(400, "Missing request body".to_owned()));
        }
    };
    let request: add_subscription_request::AddSubscriptionRequest =
        match serde_json::from_str(&body) {
            Ok(request) => request,
            Err(_error) => {
                return Ok(create_response(400, "Malformed request body".to_owned()));
            }
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

    let html_content = include_str!("verification_email.html");
    let email_request = SendEmailRequest {
        from: From {
            name: "Göteborg Farligt Avfall Notifications".to_owned(),
            email: format!("noreply-farligtavfall@{}", email_domain),
        },
        subject: "Please verify your subscription".to_owned(),
        recipients: vec![Recipient {
            email: subscription.email,
            substitutions: [("-verifyUrl-".to_owned(), format!("{}?auth_token={}", verify_url, subscription.auth_token.unwrap()))].iter()
                .cloned()
                .collect::<HashMap<String, String>>()
        }],
        html_content: html_content.to_owned()
    };
    match send_email(&api_key, email_request).await {
        Ok(_response) => Ok(create_response(200, "Successfully created subscription".to_owned())),
        Err(error) => {
            error!("Failed to send verification email: {}", error);
            Ok(create_response(500, "Failed to send verification email".to_owned()))
        }
    }
}

fn create_response(status_code: i64, body: String) -> ApiGatewayProxyResponse {
    ApiGatewayProxyResponse {
        status_code,
        headers: HashMap::new(),
        multi_value_headers: HashMap::new(),
        body: Some(body),
        is_base64_encoded: None,
    }
}
