use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(fmt::Debug)]
#[derive(Serialize)]
#[derive(Deserialize)]
pub struct Coordinate {
    latitude: String,
    longitude: String,
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {}\n", self.latitude, self.longitude)
    }
}

