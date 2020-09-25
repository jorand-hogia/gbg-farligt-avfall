use std::fmt;
use chrono::{DateTime, FixedOffset, Utc, TimeZone, Datelike};
use chrono_tz::Europe::Stockholm;
use regex::Regex;
use select::{document, predicate, selection, node};

pub struct PickUpEvent {
    street: String,
    district: String,
    description: Option<String>,
    time: DateTime<FixedOffset>,
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
        let other_stuff = match node.find(predicate::Class("c-snippet__section"))
            .into_selection().first() {
                Some(element) => element.text(),
                None => {
                    println!("Not found :(");
                    continue;
                }
            };
        let (description, raw_times) = split_desc_and_times(other_stuff).unwrap();
        let utc = Utc::now().naive_utc();
        let current_year = Stockholm.from_utc_datetime(&utc).year();
        let times: Vec<DateTime::<FixedOffset>> = match parse_times(&raw_times, current_year) {
            Ok(times) => times,
            Err(e) => return Err(PageParserError{
                message: e.message
            })
        };
        println!("{}", raw_times);
    }
    Ok(Vec::new())
} 

fn format_street(raw: String) -> String {
    String::from(raw.trim())
}

fn format_district(raw: String) -> String {
    String::from(raw.replace("Kommunal,", "").trim())
}

fn split_desc_and_times(raw: String) -> Result<(Option<String>, String), PageParserError> {
    let raw = raw.trim().to_lowercase();
    let re = Regex::new("måndag|tisdag|onsdag|torsdag|fredag|lördag|söndag").unwrap();
    let result = match re.find(&raw) {
        Some(res) => res.start(),
        None => return Err(PageParserError{
            message: format!("Could not find a swedish day name. This input is fucked :(")
        }) 
    };
    let description = match result == 0 {
        true => None,
        false => Some(String::from(String::from(&raw[0..result]).replace(".", "").trim()))
    };
    let raw_times = String::from(String::from(&raw[result..]).trim_matches('.').trim()); // TODO: Do something about this crap when you learn how to
    Ok((description, raw_times))
}

fn parse_times(raw: &String, year: i32) -> Result<Vec<DateTime::<FixedOffset>>, PageParserError> {
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
    fn should_split_description_and_times() {
        let raw = "
                                        Vid pizzerian. Tisdag 6 oktober 19-19.45.

                                    ";
        let (description, raw_times) = split_desc_and_times(String::from(raw)).unwrap();
        assert_eq!(true, description.is_some());
        assert_eq!("vid pizzerian", description.unwrap());
        assert_eq!("tisdag 6 oktober 19-19.45", raw_times);
    }

    #[test]
    fn should_parse_single_date() {
        let raw = "måndag 28 september 17-17.45";
        let time = parse_times(&raw.to_string(), 2020 as i32).unwrap();
        assert_eq!(1, time.len());
        assert_eq!(DateTime::parse_from_rfc3339("2020-09-28T15:00:00Z").unwrap(), *time.get(0).unwrap())
    }

    #[test]
    fn temp_test() {
        let file = read_file("body_with_items.html");
        parse_page(file);
    }
}