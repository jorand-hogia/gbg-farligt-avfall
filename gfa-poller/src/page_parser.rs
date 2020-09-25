use std::fmt;
use chrono::{DateTime, Utc};
use select::{document, predicate, selection, node};

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
    let doc = match document::Document::from_read(page.as_slice()) {
        Ok(doc) => doc,
        Err(_e) => return Err(PageParserError{
            message: format!("Could not parse HTML document")
        })
    };
    for node in doc.find(predicate::Class("c-snippet")) {
        let street = match node.find(predicate::Class("c-snippet__title"))
            .into_selection().children().first() {
                Some(element) => format_street(element.text()), 
                None => {
                    println!("Not found :(");
                    continue;
                }
            };
        let district = match node.find(predicate::Class("c-snippet__meta"))
            .into_selection().first() {
                Some(element) => format_district(element.text()),
                None => {
                    println!("Not found :(");
                    continue; // TODO: Proper log statements
                }
            };
        println!("{} - {}", district, street);
    }
    Ok(Vec::new())
} 

fn format_street(raw: String) -> String {
    String::from(raw.trim())
}

fn format_district(raw: String) -> String {
    String::from(raw.replace("Kommunal,", "").trim())
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

    #[test]
    fn should_format_street() {
        let raw_street = "
                                    Bankebergsgatan/Kennedygatan";
        let formatted_street = format_street(String::from(raw_street));
        assert_eq!("Bankebergsgatan/Kennedygatan", formatted_street);
    }

    #[test]
    fn should_format_district() {
        let raw_district = "Kommunal, Västra Göteborg";
        let formatted_district = format_district(String::from(raw_district));
        assert_eq!("Västra Göteborg", formatted_district);
    }

    #[test]
    fn temp_test() {
        let file = read_file("body_with_items.html");
        parse_page(file);
    }
}