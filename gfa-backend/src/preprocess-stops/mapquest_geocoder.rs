use std::error::Error;
use std::fmt;
use std::collections::HashMap;
use reqwest::blocking::{Client};
use serde_json::{Value, json, from_str};
use common::coordinate::Coordinate;
use crate::geocoder::GeoCoder;

pub struct MapQuestGeoCoder {}

#[derive(Debug)]
pub struct MapQuestGeoCoderError {
    message: String 
}
impl Error for MapQuestGeoCoderError {
    fn description(&self) -> &str {
        &self.message
    }
}
impl fmt::Display for MapQuestGeoCoderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n", self.message)
    }
}

impl GeoCoder for MapQuestGeoCoder {

    fn forward_geocode(api_key: String, id_by_address: HashMap<String, String>) -> Result<HashMap<String, Coordinate>, Box<dyn Error>> {
        let client = Client::builder()
            .use_rustls_tls()
            .build()
            .unwrap();
        let addresses: Vec<&String> = id_by_address
            .keys()
            .collect();
        for chunk in addresses.chunks(100) {
            let chunk = chunk.to_vec();
            let location_params = location_params(chunk);
            let response_raw = client.get("http://open.mapquestapi.com/geocoding/v1/batch")
                .query(&[("key", "2FSWyWz0ouHBnucVBWA80zsPb6K5wfwc")])
                .query(&location_params)
                .send()?
                .text()?;
            println!("{}", response_raw);
        }
        Ok(HashMap::new())
    }
}

// /batch?key=KEY&location=Denver,CO&location=Boulder,CO
fn location_params(address_by_id: Vec<&String>) -> Vec<(String, String)> {
    address_by_id.into_iter()
        .map(|address_by_id| {
            ("location".to_string(), address_by_id.clone())
        })
        .collect()
}

fn response_to_coordinates(response: String) -> Result<HashMap<String, Coordinate>, Box<dyn Error>> {
    let json: Value = serde_json::from_str(&response).unwrap();
    println!("{}\n\n\n\n", json);
    let res = match json.get("results") {
        Some(res) => res,
        None => return Err(Box::new(MapQuestGeoCoderError{
            message: format!("Missing key 'results' in response json")
        }))
    };
    let res = match res.as_array() {
        Some(res) => res,
        None => return Err(Box::new(MapQuestGeoCoderError{
            message: format!("Key 'results' in response json could not be parsed as array")
        }))
    };
    for result in res {
        let address = match result.get("providedLocation") {
            Some(providedLocation) => providedLocation,
            None => return Err(Box::new(MapQuestGeoCoderError{
                message: format!("Result contained no 'providedLocation")
            }))
        };
        let address = match address.get("location") {
            Some(address) => address,
            None => return Err(Box::new(MapQuestGeoCoderError{
                message: format!("Result contained no 'location'")
            }))
        };
        println!("{}", address);
    }
    Ok(HashMap::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn read_file(path: &str) -> String {
        fs::read_to_string(format!("{}/src/preprocess-stops/resources/test/{}", env!("CARGO_MANIFEST_DIR"), path)).unwrap()
    }

    fn temp() {
        let mut addresses: HashMap::<String, String> = HashMap::new();
        addresses.insert("Järntorget, Göteborg".to_string(), "some-id".to_string());
        addresses.insert("Brunnsparken, Göteborg".to_string(), "some-id-2".to_string());
        MapQuestGeoCoder::forward_geocode("key".to_string(), addresses);
    }

    #[test]
    fn should_convert_location_params() {
        let mut input: Vec<&String> = Vec::new();
        let some_address = "Some Gata, Hisingen".to_string();
        input.push(&some_address);
        let location_params = location_params(input);
        assert_eq!(1, location_params.len());
        assert_eq!(&"Some Gata, Hisingen".to_string(), &location_params.get(0).unwrap().1);
    }

    #[test]
    fn should_convert_response() {
        let response = read_file("response.json");
        let coordinates = response_to_coordinates(response).unwrap();
        assert_eq!(true, coordinates.contains_key("Järntorget,Göteborg"));
        assert_eq!(true, coordinates.contains_key("Brunnsparken"))
    }
}