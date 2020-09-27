use std::fmt;
use chrono::{DateTime, FixedOffset, Utc, TimeZone, Datelike};
use chrono_tz::Europe::Stockholm;
use regex::{Regex, CaptureNames};
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
    let datetimes: Vec::<DateTime::<FixedOffset>> = Vec::new();
    for dt in raw.split_terminator("och") {
        let dt = append_zeros_in_timestamp(dt); 
        let re = Regex::new(r"\w+ (?P<day>\d{1,2}) (?P<month>\w+) (?P<start>\d{2}\.\d{2})-(?P<end>\d{2}\.\d{2})").unwrap();
        let captures = match re.captures(&dt) {
            Some(caps) => caps,
            None => return Err(PageParserError{
                message: format!("Could not parse timestamp: {}", dt)
            })
        };
        let day = match captures.name("day") {
            Some(day) => day.as_str(),
            None => return Err(PageParserError{
                message: format!("Missing day in timestamp: {}", dt)
            })
        };
        let month = match captures.name("month") {
            Some(month) => month.as_str(),
            None => return Err(PageParserError{
                message: format!("Missing month in timestamp: {}", dt)
            })
        };
        let start_time = match captures.name("start") {
            Some(start) => start.as_str(),
            None => return Err(PageParserError{
                message: format!("Missing start time in timestamp: {}", dt)
            })
        };
        let end_time = match captures.name("end") {
            Some(end) => end.as_str(),
            None => return Err(PageParserError{
                message: format!("Missing end time in timestamp: {}", dt)
            })
        };
        println!("{} - {}", dt, year);
        println!("Day: {}, Month: {}, Start time: {}, End time: {}", day, month, start_time, end_time);
    }
    Ok(datetimes)
}

fn append_zeros_in_timestamp(raw: &str) -> String {
    let dt = raw.trim();
    let re = Regex::new(r"[^\.](?P<bad_hour>\d{2})-").unwrap();
    let dt = re.replace(dt, " $bad_hour.00-");
    String::from(dt)
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
    fn should_append_zeros_to_timestamp() {
        let raw = " torsdag 29 oktober 17-17.20";
        assert_eq!("torsdag 29 oktober 17.00-17.20".to_string(), append_zeros_in_timestamp(&raw))
    }

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