use std::fmt;
use sha1::{Sha1};
use rand::prelude::*;
use chrono::{Duration, Utc};

#[derive(fmt::Debug)]
pub struct Subscription {
    pub email: String,
    pub location_id: String,
    pub auth_token: String,
    pub is_authenticated: bool,
    pub ttl: Option<i64>
}

impl Subscription {
    pub fn new(email: String, location_id: String) -> Self {
        let mut random_bytes = [0u8; 20];
        thread_rng().fill_bytes(&mut random_bytes);
        let mut auth_token = Sha1::new();
        auth_token.update(&random_bytes);
        auth_token.update(email.clone().as_bytes());
        auth_token.update(location_id.clone().as_bytes());

        Subscription{
            email: email,
            location_id: location_id,
            auth_token: auth_token.digest().to_string(),
            is_authenticated: false,
            ttl: Some((Utc::now() + Duration::days(1)).timestamp())
        }
    }
    pub fn verify(&mut self) {
        self.is_authenticated = true;
        self.ttl = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_subscription() {
        let subscription = Subscription::new("email@email.com".to_owned(), "hisingen_nice".to_owned());
        assert_eq!("email@email.com".to_owned(), subscription.email);
        assert_eq!("hisingen_nice".to_owned(), subscription.location_id);
        assert_eq!(false, subscription.is_authenticated);
    }
}
