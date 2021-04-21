use std::{error, fmt};
use reqwest::{Client, StatusCode};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Serialize;

pub struct SendEmailRequest {
    subject: String,
    html_content: String,
    from: From,
    recipients: Vec<String>
}
#[derive(Serialize, Debug)]
pub struct From {
    name: String,
    email: String,
}
#[derive(Debug)]
pub struct BadStatusCode {
  status_code: StatusCode
}
impl fmt::Display for BadStatusCode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Bad status code: {}", self.status_code)
  } 
}
impl error::Error for BadStatusCode {}

#[derive(Serialize, Debug)]
struct RequestBody {
  personalizations: Vec<Personalization>,
  from: From,
  subject: String,
  content: Vec<Content>
}
#[derive(Serialize, Debug)]
struct Personalization {
  to: Vec<To>
}
#[derive(Serialize, Debug)]
struct To {
  email: String
}
#[derive(Serialize, Debug)]
struct Content {
  #[serde(rename = "type")]
  content_type: String,
  value: String,
}

const URL: &str = "https://api.sendgrid.com/v3/mail/send";

pub async fn send_email(api_key: String, request: SendEmailRequest) -> Result<(), Box<dyn error::Error>> {
    let client = Client::builder()
        .use_rustls_tls()
        .build()
        .unwrap();
        HeaderName::from_lowercase(b"authorization").unwrap();
    let mut headers = HeaderMap::new();
    headers.insert(
      HeaderName::from_lowercase(b"authorization").unwrap(),
      HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap()
    );
    match client.post(URL)
      .body(create_request_body(request))
      .headers(headers)
      .send()
      .await {
        Ok(res) => {
          if res.status().is_success() {
            return Ok(());
          }
          return Err(Box::new(BadStatusCode{
            status_code: res.status()
          }));
        },
        Err(e) => Err(Box::new(e))
      }
}

fn create_request_body(request: SendEmailRequest) -> String {
  let request_body = RequestBody{
    from: request.from,
    subject: request.subject,
    content: vec![Content{
      content_type: "text/html".to_owned(),
      value: request.html_content
    }],
    personalizations: vec![Personalization{
      to: request.recipients
        .iter()
        .map(|email| To{
          email: email.clone(),
        })
        .collect()
    }]
  };
  serde_json::to_string(&request_body).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_proper_request_body() {
      let send_email_request = SendEmailRequest{
        subject: "My subject".to_owned(),
        html_content: "<p>I are HTML content</p>".to_owned(),
        from: From{
          name: "My sender name".to_owned(),
          email: "my-sender-email@some-domain.com".to_owned()
        },
        recipients: vec!["first@email.com".to_owned(), "second@email.com".to_owned()]
      };
      let expected_response_body = "{\
        \"personalizations\":[\
          {\
            \"to\":[\
              {\
                \"email\":\"first@email.com\"\
              },\
              {\
                \"email\":\"second@email.com\"\
              }\
            ]\
          }\
        ],\
        \"from\":{\
          \"name\":\"My sender name\",\
          \"email\":\"my-sender-email@some-domain.com\"\
        },\
        \"subject\":\"My subject\",\
        \"content\":[\
          {\
            \"type\":\"text/html\",\
            \"value\":\"<p>I are HTML content</p>\"\
          }\
        ]\
      }".to_owned();
      assert_eq!(expected_response_body, create_request_body(send_email_request));
    }
}
