use std::fmt;

#[derive(fmt::Debug)]
pub struct PickUpEvent {
    pub street: String,
    pub district: String,
    pub description: Option<String>,
    pub time_start: String,
    pub time_end: String,
}

impl fmt::Display for PickUpEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {} ({}): {} to {}\n", self.district, self.street, self.description.as_ref().unwrap_or(&"-".to_string()), self.time_start, self.time_end)
    }
}

