use std::fmt;
use chrono::{DateTime, Utc};

pub struct PickUpEvent {
    street: String,
    district: String,
    description: Option<String>,
    time: DateTime<Utc>,
}

pub struct PageParserError {
    message: String,
}
impl fmt::Debug for PageParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ file: {}, line: {} }}", file!(), line!())
    }
}

fn parse_page(page: Vec<u8>) -> Result<Vec<PickUpEvent>, PageParserError> {
    Ok(Vec::new())
} 

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{File, metadata};
    use std::io::Read;

    fn read_file(path: &str) -> Vec<u8> {
        let path = &format!("{}/src/resources/test/{}", env!("CARGO_MANIFEST_DIR"), path);
        let mut file = File::open(path).unwrap();
        let md = metadata(&path).unwrap(); 
        let mut buffer = vec![0; md.len() as usize];
        file.read(&mut buffer).unwrap();
        buffer
    }
}