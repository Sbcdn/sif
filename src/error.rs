use std::fmt::{Display,Formatter,Result as fmtResult};
use std::error::Error;

#[derive(Debug, Clone)]
pub struct SifError{
    details: String
}

impl SifError {
    pub fn new(msg: &str) -> SifError {
        SifError {details : msg.to_string() }
    }
}

impl Display for SifError {
    fn fmt(&self, f: &mut Formatter) -> fmtResult {
        write!(f,"{}",self.details)
    }
}

impl Error for SifError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl From<hex::FromHexError> for SifError {
    fn from(err: hex::FromHexError) -> Self {
        SifError::new(&err.to_string())
    }
}

impl From<cardano_serialization_lib::error::DeserializeError> for SifError {
    fn from(err: cardano_serialization_lib::error::DeserializeError) -> Self {
        SifError::new(&err.to_string())
    }
}

impl From<cardano_serialization_lib::error::JsError> for SifError {
    fn from(err: cardano_serialization_lib::error::JsError) -> Self {
        SifError::new(&err.to_string())
    }
}

impl From<minreq::Error> for SifError {
    fn from(err: minreq::Error) -> Self {
        SifError::new(&err.to_string())
    }
}

impl From<std::io::Error> for SifError {
    fn from(err: std::io::Error) -> Self {
        SifError::new(&err.to_string())
    }
}

impl From<serde_json::Error> for SifError {
    fn from(err: serde_json::Error) -> Self {
        SifError::new(&err.to_string())
    }
}

