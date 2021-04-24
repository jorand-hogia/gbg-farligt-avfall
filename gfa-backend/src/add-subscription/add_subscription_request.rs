use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(fmt::Debug, Serialize, Deserialize)]
pub struct AddSubscriptionRequest {
    pub email: String,
    pub location_id: String
}
