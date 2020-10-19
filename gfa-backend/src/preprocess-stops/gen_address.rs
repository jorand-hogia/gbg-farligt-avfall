use lazy_static::lazy_static;
use regex::{Regex};

pub fn generate_address(street: &String, district: &String) -> String {
    lazy_static! {
        static ref MULTIPLE_STREETS: Regex = Regex::new(r"^(.+)/(.+)$").unwrap();
        static ref WITH_STREET_AS_SECONDARY: Regex = Regex::new(r"^(.+), (.+)$").unwrap();
    }
    match MULTIPLE_STREETS.captures(street) {
        Some(result) => {
            let first_street = result.get(1).unwrap().as_str();
            return format!("{},{},Göteborg", first_street, district);
        },
        None => {}
    };
    match WITH_STREET_AS_SECONDARY.captures(street) {
        Some(result) => {
            let street = result.get(2).unwrap().as_str();
            return format!("{},{},Göteborg", street, district);
        },
        None => {}
    };
    return format!("{},{},Göteborg", street, district);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_split_multiple_streets() {
        let address = generate_address(&"Kristinelundsgatan/Chalmersgatan".to_string(), &"Centrum".to_string());
        assert_eq!("Kristinelundsgatan,Centrum,Göteborg".to_string(), address);
    }

    #[test]
    fn should_get_street_where_street_is_secondary() {
        let address = generate_address(&"Kaserntorget, Kungsgatan 22".to_string(), &"Centrum".to_string());
        assert_eq!("Kungsgatan 22,Centrum,Göteborg".to_string(), address);
    }

    #[test]
    fn should_not_modify_good_addresses() {
        let address = generate_address(&"Framnäsgatan 31A".to_string(), &"Centrum".to_string());
        assert_eq!("Framnäsgatan 31A,Centrum,Göteborg".to_string(), address);
        let address = generate_address(&"Lövgärdets Centrum".to_string(), &"Angered".to_string());
        assert_eq!("Lövgärdets Centrum,Angered,Göteborg", address);
    }
}