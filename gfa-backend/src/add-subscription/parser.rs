use crate::add_subscription_request::AddSubscriptionRequest;
use aws_lambda_events::event::apigw::ApiGatewayV2httpRequest;

pub struct ParseError {
    pub status_code: i64,
    pub message: String
}

pub fn parse_request(event: ApiGatewayV2httpRequest) -> Result<AddSubscriptionRequest, ParseError> {
    let body = match event.body {
        Some(body) => body,
        None => {
            return Err(ParseError{status_code: 400, message: "Missing request body".to_owned()})
        }
    };
    match serde_json::from_str(&body) {
        Ok(request) => Ok(request),
        Err(_error) => {
            Err(ParseError{status_code: 400, message: "Malformed request body".to_owned()})
        }
    }
}
