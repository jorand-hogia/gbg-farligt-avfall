use std::{fmt::Debug};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Subscription {
    email: String,
    location_id: String,
} 
