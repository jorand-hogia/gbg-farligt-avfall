use std::fmt;
use chrono::{DateTime, FixedOffset, Utc, TimeZone, Datelike};
use chrono_tz::Europe::Stockholm;
use regex::{Regex, CaptureNames};
use select::{document, predicate, selection, node};

pub struct PickUpEvent {
    street: String,
    district: String,
    description: Option<String>,
    time_start: String,
    time_end: String, 
}
impl fmt::Display for PickUpEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {} ({}): {}-{}\n", self.district, self.street, self.description.as_ref().unwrap_or(&"-".to_string()), self.time_start, self.time_end)
    }
}

pub struct PageParserError {
    pub message: String,
}
impl fmt::Debug for PageParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ file: {}, line: {} }}", file!(), line!())
    }
}

pub fn parse_page(page: Vec<u8>) -> Result<Vec<PickUpEvent>, PageParserError> {
    let doc = match document::Document::from_read(page.as_slice()) {
        Ok(doc) => doc,
        Err(_e) => return Err(PageParserError{
            message: format!("Could not parse HTML document")
        })
    };
    let mut events: Vec::<PickUpEvent> = Vec::new();
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
        let (description, raw_times) = match split_desc_and_times(other_stuff) {
            Ok(result) => result,
            Err(e) => {
                println!("{}", e.message);
                continue;
            }
        };
        let utc = Utc::now().naive_utc();
        let current_year = Stockholm.from_utc_datetime(&utc).year();
        let times: Vec<(DateTime::<chrono_tz::Tz>, DateTime<chrono_tz::Tz>)> = match parse_times(&raw_times, current_year) {
            Ok(times) => times,
            Err(e) => {
                println!("{}", e.message);
                continue;
            }
        };
        for t in times {
            events.push(PickUpEvent{
                street: String::from(&street),
                district: String::from(&district),
                description: description.clone(),
                time_start: t.0.to_rfc3339(),
                time_end: t.1.to_rfc3339(),
            })
        }
    }
    Ok(events)
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

fn parse_times(raw: &String, year: i32) -> Result<Vec<(DateTime::<chrono_tz::Tz>, DateTime<chrono_tz::Tz>)>, PageParserError> {
    let mut datetimes: Vec::<(DateTime::<chrono_tz::Tz>, DateTime<chrono_tz::Tz>)> = Vec::new();
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
        let day = zero_pad_day_number(day);
        let month = match captures.name("month") {
            Some(month) => month.as_str(),
            None => return Err(PageParserError{
                message: format!("Missing month in timestamp: {}", dt)
            })
        };
        let month = month_to_english(month)?;
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
        let start = format!("{}-{}-{} {}", year, month, day, start_time);
        let end = format!("{}-{}-{} {}", year, month, day, end_time);
        let start = match Stockholm.datetime_from_str(&start, "%Y-%B-%d %H.%M") {
            Ok(res) => res,
            Err(_e) => return Err(PageParserError{
                message: format!("Could not parse timestamp: {}", start)
            })
        };
        let end = match Stockholm.datetime_from_str(&end, "%Y-%B-%d %H.%M") {
            Ok(res) => res,
            Err(_e) => return Err(PageParserError{
                message: format!("Could not parse timestamp: {}", start)
            })
        };
        datetimes.push((start, end));
    }
    Ok(datetimes)
}

fn append_zeros_in_timestamp(raw: &str) -> String {
    let dt = raw.trim();
    let re = Regex::new(r"[^\.](?P<bad_hour>\d{2})-").unwrap();
    let dt = re.replace(dt, " $bad_hour.00-");
    String::from(dt)
}

fn zero_pad_day_number(raw: &str) -> String {
    let re = Regex::new(r"^(?P<day_number>\d{1})$").unwrap();
    let dt = re.replace(raw, "0$day_number");
    String::from(dt)
}

fn month_to_english(swe_month: &str) -> Result<String, PageParserError> {
    match swe_month {
        "januari" => Ok(String::from("january")),
        "februari" => Ok(String::from("february")),
        "mars" => Ok(String::from("march")),
        "april" => Ok(swe_month.to_string()),
        "maj" => Ok(String::from("may")),
        "juni" => Ok(String::from("june")),
        "juli" => Ok(String::from("july")),
        "augusti" => Ok(String::from("august")),
        "september" => Ok(swe_month.to_string()),
        "sepetmber" => Ok(String::from("september")),
        "oktober" => Ok(String::from("october")),
        "november" => Ok(swe_month.to_string()),
        "december" => Ok(swe_month.to_string()),
        _ => return Err(PageParserError{
            message: format!("Invalid month name: {}", swe_month)
        })
    }
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

    #[test]
    fn should_zero_pad_single_digit_day() {
        let raw = "1";
        assert_eq!("01".to_string(), zero_pad_day_number(raw))
    }

    #[test]
    fn should_not_zero_pad_double_digit_day() {
        let raw = "29";
        assert_eq!("29".to_string(), zero_pad_day_number(raw))
    }

    #[test]
    fn should_handle_month_misspellings() {
        assert_eq!("september".to_string(), month_to_english(&"sepetmber".to_string()).unwrap());
    }

    #[test]
    fn should_parse_single_event() {
        let raw = "Måndag 28 september 17-17.45";
        let time = parse_times(&raw.to_string(), 2020 as i32).unwrap();
        assert_eq!(1, time.len());
        assert_eq!("2020-09-28T17:00:00+02:00".to_string(), time.get(0).unwrap().0.to_rfc3339());
        assert_eq!("2020-09-28T17:45:00+02:00", time.get(0).unwrap().1.to_rfc3339());
    }

    #[test]
    fn should_handle_daylight_saving() {
        let with_dst = "Tisdag 24 oktober 16-16.20";
        let without_dst = "Måndag 25 oktober 16-16.20";
        assert_eq!("2020-10-24T16:00:00+02:00", parse_times(&with_dst.to_string(), 2020).unwrap().get(0).unwrap().0.to_rfc3339());
        assert_eq!("2020-10-25T16:00:00+01:00", parse_times(&without_dst.to_string(), 2020).unwrap().get(0).unwrap().0.to_rfc3339());
    }

    #[test]
    fn should_parse_multiple_events() {
        let raw = "Torsdag 17 september 17-17.20 och torsdag 20 oktober 17-17.20";
        let times = parse_times(&raw.to_string(), 2020).unwrap();
        assert_eq!(2, times.len());
        assert_eq!("2020-10-20T17:00:00+02:00".to_string(), times.get(1).unwrap().0.to_rfc3339());
        assert_eq!("2020-10-20T17:20:00+02:00".to_string(), times.get(1).unwrap().1.to_rfc3339());
    }

    #[test]
    fn should_parse_full_page() {
        let file = read_file("body_with_items.html");
        let events = parse_page(file).unwrap();
        assert_eq!(39, events.len());
        for e in events {
            println!("{}", e);
        }
    }
}