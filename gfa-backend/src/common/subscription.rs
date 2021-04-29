use std::fmt;
use sha2::{Sha512, Digest};
use rand::prelude::*;
use chrono::{Duration, Utc};

#[derive(fmt::Debug)]
pub struct Subscription {
    pub email: String,
    pub location_id: String,
    pub auth_token: Option<String>,
    pub is_authenticated: bool,
    pub ttl: Option<i64>
}

impl Subscription {
    pub fn new(email: String, location_id: String) -> Self {
        let mut random_bytes = [0u8; 32];
        thread_rng().fill_bytes(&mut random_bytes);
        let mut auth_token = Sha512::new();
        auth_token.update(&random_bytes);
        auth_token.update(email.as_bytes());
        auth_token.update(location_id.as_bytes());

        Subscription{
            email,
            location_id,
            auth_token: Some(format!("{:x}", auth_token.finalize())),
            is_authenticated: false,
            ttl: Some((Utc::now() + Duration::days(1)).timestamp())
        }
    }
    pub fn verify(&mut self) {
        self.is_authenticated = true;
        self.ttl = None;
        self.auth_token = None;
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
