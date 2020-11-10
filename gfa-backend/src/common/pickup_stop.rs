use std::fmt;
use serde::{Serialize, Deserialize};
use crate::coordinate::Coordinate;

#[derive(fmt::Debug, Serialize, Deserialize)]
pub struct PickUpStop {
    pub location_id: String,
    pub street: String,
    pub district: String,
    pub description: Option<String>,
    pub coordinate: Option<Coordinate>,
}

impl fmt::Display for PickUpStop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {} ({}) ({})\n",
            self.district,
            self.street,
            match &self.description {
                Some(description) => description.clone(),
                None => "-".to_string()
            },
            match &self.coordinate {
                Some(coordinate) => format!("{}", coordinate),
                None => "-".to_string()
            },
        )
    }
}

impl PartialEq for PickUpStop {
    fn eq(&self, other: &Self) -> bool {
        self.location_id == other.location_id
    }
}

impl PickUpStop {
    pub fn new(location_id: String, street: String, district: String, description: Option<String>) -> Self {
        PickUpStop{
            location_id,
            street,
            district,
            description,
            coordinate: None
        }
    }
    
    pub fn from(stop: &PickUpStop, coordinate: Option<Coordinate>) -> PickUpStop {
        PickUpStop{
            location_id: stop.location_id.clone(),
            street: stop.street.clone(),
            district: stop.district.clone(),
            description: stop.description.clone(),
            coordinate: coordinate
        }
    }

    pub fn set_coordinate(mut self, coordinate: Option<Coordinate>) {
        self.coordinate = coordinate;
    }
}
