use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(fmt::Debug, Serialize, Deserialize)]
pub struct PickUpStop {
    pub location_id: String,
    pub street: String,
    pub district: String,
    pub description: Option<String>,
}

impl fmt::Display for PickUpStop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{} - {} ({})",
            self.district,
            self.street,
            match &self.description {
                Some(description) => description.clone(),
                None => "-".to_owned()
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
        }
    }
}
