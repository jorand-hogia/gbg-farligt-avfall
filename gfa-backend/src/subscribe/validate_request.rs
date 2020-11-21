use std::{error};
use crate::subscription::Subscription;

pub fn validate(subscription: &Subscription) -> Result<(), Box<dyn error::Error>> {
    // TODO: validate email format
    // TODO: validate location-id exists?
    Ok(())
}
