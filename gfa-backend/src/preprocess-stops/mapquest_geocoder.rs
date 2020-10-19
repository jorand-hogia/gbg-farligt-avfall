use std::collections::HashMap;
use reqwest::blocking::{Client};
use serde_json::Value;
use serde::{Deserialize};
use common::coordinate::Coordinate;
use crate::geocoder::GeoCoder;

pub struct MapQuestGeoCoder {}

#[derive(Deserialize)]
struct ApiResponse {
    results: Vec<ApiResult>
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiResult {
    provided_location: ApiProvidedLocation,
    locations: Vec<ApiLocation>
}

#[derive(Deserialize)]
struct ApiProvidedLocation {
    location: String
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiLocation {
    lat_lng: ApiLatLng
}

#[derive(Deserialize)]
struct ApiLatLng {
    lat: f64,
    lng: f64
}

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

impl GeoCoder for MapQuestGeoCoder {

    fn forward_geocode(api_key: String, id_by_address: HashMap<String, String>) -> Result<HashMap<String, Option<Coordinate>>, Error> {
        let client = Client::builder()
            .use_rustls_tls()
            .build()
            .unwrap();
        let addresses: Vec<&String> = id_by_address
            .keys()
            .collect();
        let mut coordinate_by_id: HashMap<String, Option<Coordinate>> = HashMap::new();
        for chunk in addresses.chunks(100) {
            let chunk = chunk.to_vec();
            let location_params = location_params(chunk);
            let request = client.get("http://mapquestapi.com/geocoding/v1/batch")
                .query(&[("key", &api_key), ("inFormat", &"kvp".to_string()), ("outFormat", &"json".to_string()), ("thumbsMap", &"false".to_string())])
                .query(&location_params);
            println!("REQUEST PATH: {:?}", request);
            let response_raw = request
                .send()?
                .text()?;
            let coordinates_by_address = response_to_coordinates(response_raw)?;
            for (address, coordinate) in coordinates_by_address.iter() {
                let identifier = id_by_address.get(address).unwrap();
                coordinate_by_id.insert(identifier.clone(), coordinate.clone());
            }
        }
        Ok(coordinate_by_id)
    }
}

fn location_params(address_by_id: Vec<&String>) -> Vec<(String, String)> {
    address_by_id.into_iter()
        .map(|address_by_id| {
            ("location".to_string(), address_by_id.clone())
        })
        .collect()
}

fn response_to_coordinates(response: String) -> Result<HashMap<String, Option<Coordinate>>, Error> {
    let json: Value = serde_json::from_str(&response)?;
    let api_response: ApiResponse = serde_json::from_value(json)?;
    let mut coordinates_by_address: HashMap<String, Option<Coordinate>> = HashMap::new();
    for result in api_response.results {
        let provided_address = result.provided_location.location;
        if result.locations.is_empty() {
            coordinates_by_address.insert(provided_address.clone(), None);
            continue;
        }
        let first_location = result.locations.first().unwrap();
        coordinates_by_address.insert(provided_address.clone(), Some(Coordinate::new(
            first_location.lat_lng.lat,
            first_location.lat_lng.lng
        )));
    }
    Ok(coordinates_by_address)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use assert_approx_eq::assert_approx_eq;

    fn read_file(path: &str) -> String {
        fs::read_to_string(format!("{}/src/preprocess-stops/resources/test/{}", env!("CARGO_MANIFEST_DIR"), path)).unwrap()
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
        let keys: Vec<&String> = coordinates.keys().collect();
        for key in keys {
            println!("{}", key);
        }
        assert_eq!(true, coordinates.contains_key("Järntorget,Göteborg"));
        let first_coord = coordinates.get("Järntorget,Göteborg").unwrap().as_ref().unwrap();
        assert_approx_eq!(57.700072, first_coord.latitude());
        assert_approx_eq!(11.951992, first_coord.longitude());
        assert_eq!(true, coordinates.contains_key("Brunnsparken,Göteborg"));
        let second_coord = coordinates.get("Brunnsparken,Göteborg").unwrap().as_ref().unwrap();
        assert_approx_eq!(57.706784, second_coord.latitude());
        assert_approx_eq!(11.969905, second_coord.longitude());
    }
}