use std::{fmt::Debug};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Subscription {
    pub email: String,
    pub location_id: String,
} 
