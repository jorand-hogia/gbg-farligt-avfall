use std::fmt;
use std::result::Result;
use chrono::{DateTime, Utc, TimeZone, Datelike};
use chrono_tz::Europe::Stockholm;
use regex::{Regex};
use select::{document, predicate};

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

#[derive(fmt::Debug)]
pub struct PageParserError {
    pub message: String,
}
impl fmt::Display for PageParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub fn parse_page(page: Vec<u8>) -> Result<Vec<PickUpEvent>, Vec<PageParserError>> {
    let mut errors: Vec::<PageParserError> = Vec::new();
    let doc = match document::Document::from_read(page.as_slice()) {
        Ok(doc) => doc,
        Err(_e) => {
            errors.push(PageParserError{
                message: format!("Could not parse HTML document")
            });
            return Err(errors);
        }
    };
    let mut events: Vec::<PickUpEvent> = Vec::new();
    for node in doc.find(predicate::Class("c-snippet")) {
        let street = match node.find(predicate::Class("c-snippet__title"))
            .into_selection().children().first() {
                Some(element) => format_street(element.text()), 
                None => {
                    errors.push(PageParserError{
                        message: format!("No element with class c-snippet__title found")
                    });
                    continue;
                }
            };
        let district = match node.find(predicate::Class("c-snippet__meta"))
            .into_selection().first() {
                Some(element) => format_district(element.text()),
                None => {
                    errors.push(PageParserError{
                        message: format!("No element with class c-snippet__meta found")
                    });
                    continue;
                }
            };
        let other_stuff = match node.find(predicate::Class("c-snippet__section"))
            .into_selection().first() {
                Some(element) => element.text(),
                None => {
                    errors.push(PageParserError{
                        message: format!("No element with class c-snippet__selection found")
                    });
                    continue;
                }
            };
        let (description, raw_times) = match split_desc_and_times(other_stuff) {
            Ok(description_and_times) => description_and_times,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        let utc = Utc::now().naive_utc();
        let current_year = Stockholm.from_utc_datetime(&utc).year();
        let times: Vec<(DateTime::<chrono_tz::Tz>, DateTime<chrono_tz::Tz>)> = match parse_times(&raw_times, current_year) {
            Ok(times) => times,
            Err(e) => {
                errors.push(e);
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
    if errors.len() > 0 {
        return Err(errors);
    }
    return Ok(events);
} 

fn format_street(raw: String) -> String {
    String::from(raw.trim())
}

fn format_district(raw: String) -> String {
    String::from(raw.replace("Kommunal,", "").trim())
}

fn split_desc_and_times(raw: String) -> Result<(Option<String>, String), PageParserError> {
    let raw = raw.trim().to_lowercase();
    // Should match either "vid ica gunnilse och återvinningsstationen. onsdag 16 september 17.35-17.55" or "onsdag 16 september 17.35-17.55"
    // Index of second group indicates where to split description and time data.
    let re = Regex::new(r"([\w\s]+\. |^)(måndag|tisdag|tisadg|onsdag|torsdag|fredag|lördag|söndag)").unwrap();
    let captures = match re.captures(&raw) {
        Some(caps) => caps,
        None => return Err(PageParserError{
            message: format!("No match found while splitting description and times: {}", raw)
        })
    };
    let index = match captures.get(2) {
        Some(group) => group.start(),
        None => return Err(PageParserError{
            message: format!("Second capturing group (swedish day name) not found while splitting description and times: {}", raw)
        })
    };
    let description = match index == 0 {
        true => None,
        false => Some(String::from(&raw[0..index]).replace(".", "").trim().to_string())
    };
    let raw_times = (String::from(&raw[index..]).trim_matches('.').trim()).to_string();
    Ok((description, raw_times))
}

fn parse_times(raw: &String, year: i32) -> Result<Vec<(DateTime::<chrono_tz::Tz>, DateTime<chrono_tz::Tz>)>, PageParserError> {
    let mut datetimes: Vec::<(DateTime::<chrono_tz::Tz>, DateTime<chrono_tz::Tz>)> = Vec::new();
    for dt in raw.split_terminator("och") {
        let dt = append_zeros_in_timestamp(dt); 
        let re = Regex::new(r"\w+ (?P<day>\d{1,2}) (?P<month>\w+) (?P<start>\d{2}\.\d{2})\s{0,1}-\s{0,1}(?P<end>\d{2}\.\d{2})").unwrap();
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
                message: format!("Could not parse timestamp: {}", end)
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
    let swe_month = swe_month.trim();
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
    fn should_split_description_and_times_with_multiple_events() {
        let raw = "På parkeringen. Torsdag 17 september 18.45-19.05 och torsdag 29 oktober 18.45-19.05.";
        let (description, raw_times) = split_desc_and_times(raw.to_string()).unwrap();
        assert_eq!(true, description.is_some());
        assert_eq!("på parkeringen", description.unwrap());
        assert_eq!("torsdag 17 september 18.45-19.05 och torsdag 29 oktober 18.45-19.05", raw_times);
    }

    #[test]
    fn should_split_description_and_times_with_handled_bad_day_name() {
        let (description, raw_times) = split_desc_and_times("på parkeringen kringlekullen. tisadg 1 september 18-18.45".to_string()).unwrap();
        assert_eq!(true, description.is_some());
        assert_eq!("på parkeringen kringlekullen", description.unwrap());
        assert_eq!("tisadg 1 september 18-18.45", raw_times);
    }

    #[test]
    fn should_error_on_unknown_bad_day_name() {
        let result = split_desc_and_times("på parkeringen. tossdag 1 september 18-18.45 och torsdag 21 september 19-19.30".to_string());
        assert_eq!(true, result.is_err());
    }

    #[test]
    fn should_split_description_and_times_with_och_in_description() {
        let (description, raw_times) = split_desc_and_times("vid ica gunnilse och återvinningsstationen. onsdag 16 september 17.35-17.55 och onsdag 28 oktober 17-17.20.".to_string()).unwrap();
        assert_eq!(true, description.is_some());
        assert_eq!("vid ica gunnilse och återvinningsstationen", description.unwrap());
        assert_eq!("onsdag 16 september 17.35-17.55 och onsdag 28 oktober 17-17.20", raw_times);
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
    fn should_parse_single_timestamp() {
        let raw = "Måndag 28 september 17-17.45";
        let time = parse_times(&raw.to_string(), 2020 as i32).unwrap();
        assert_eq!(1, time.len());
        assert_eq!("2020-09-28T17:00:00+02:00".to_string(), time.get(0).unwrap().0.to_rfc3339());
        assert_eq!("2020-09-28T17:45:00+02:00", time.get(0).unwrap().1.to_rfc3339());
    }

    #[test]
    fn should_parse_multiple_timestamps() {
        let raw = "Torsdag 17 september 17-17.20 och torsdag 20 oktober 17-17.20";
        let times = parse_times(&raw.to_string(), 2020).unwrap();
        assert_eq!(2, times.len());
        assert_eq!("2020-10-20T17:00:00+02:00".to_string(), times.get(1).unwrap().0.to_rfc3339());
        assert_eq!("2020-10-20T17:20:00+02:00".to_string(), times.get(1).unwrap().1.to_rfc3339());
    }

    #[test]
    fn should_handle_timestamp_with_bad_spacing() {
        let raw = "tisdag 27 oktober 18.10- 18.30";
        let times = parse_times(&raw.to_string(), 2020).unwrap();
        assert_eq!("2020-10-27T18:10:00+01:00".to_string(), times.get(0).unwrap().0.to_rfc3339());
        assert_eq!("2020-10-27T18:30:00+01:00".to_string(), times.get(0).unwrap().1.to_rfc3339());
    }

    #[test]
    fn should_handle_daylight_saving() {
        let with_dst = "Tisdag 24 oktober 16-16.20";
        let without_dst = "Måndag 25 oktober 16-16.20";
        assert_eq!("2020-10-24T16:00:00+02:00", parse_times(&with_dst.to_string(), 2020).unwrap().get(0).unwrap().0.to_rfc3339());
        assert_eq!("2020-10-25T16:00:00+01:00", parse_times(&without_dst.to_string(), 2020).unwrap().get(0).unwrap().0.to_rfc3339());
    }

    #[test]
    fn should_parse_full_page() {
        let file = read_file("body_with_items.html");
        let events = parse_page(file).unwrap();
        assert_eq!(39, events.len());
    }

    #[test]
    fn should_return_multiple_errors() {
        let file = read_file("body_with_very_bad_content.html");
        let events = parse_page(file);
        assert_eq!(true, events.is_err());
        assert_eq!(2, events.unwrap_err().len())
    }
}
