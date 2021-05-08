use std::fmt;
use std::error;

#[derive(Debug)]
pub struct MalformedDynamoDbResponse;
impl fmt::Display for MalformedDynamoDbResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Malformed response from DynamoDb")
    }
}
impl error::Error for MalformedDynamoDbResponse {}
