use std::collections::HashMap;
use common::coordinate::Coordinate;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

/*
 * Takes a Vec of tuples, where first element is an , and second element is an address
 * Returns a HashMap with identifier as key, and coordinate as value 
 */
pub trait GeoCoder {
    fn forward_geocode(api_key: String, id_by_address: HashMap<String, String>) -> Result<HashMap<String, Option<Coordinate>>, Error>;
}
