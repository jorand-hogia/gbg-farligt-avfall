use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, fmt::Debug)]
pub struct Coordinate {
    latitude: f64,
    longitude: f64,
}

impl Coordinate {
    pub fn new(lat: f64, lng: f64) -> Coordinate {
        Coordinate{
            latitude: lat,
            longitude: lng,
        }
    }

    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    pub fn longitude(&self) -> f64 {
        self.longitude
    }
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}, {}", self.latitude, self.longitude)
    }
}

