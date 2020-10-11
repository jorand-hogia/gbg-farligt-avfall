use std::fmt;

#[derive(fmt::Debug)]
pub struct PickUpEvent {
    street: String,
    district: String,
    description: Option<String>,
    time_start: String,
    time_end: String,
}

impl fmt::Display for PickUpEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {} ({}): {} to {}\n", self.district, self.street, self.description.as_ref().unwrap_or(&"-".to_string()), self.time_start, self.time_end)
    }
}

impl PickUpEvent {
    pub fn new(street: String, district: String, description: Option<String>, time_start: String, time_end: String) -> Self {
        PickUpEvent{
            street,
            district,
            description,
            time_start,
            time_end
        }
    }
}
