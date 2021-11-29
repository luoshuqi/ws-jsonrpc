use serde::Deserialize;
use serde_json::{to_string, Value};
use std::fmt::{Display, Formatter};

use crate::response::{Error, Response};

#[derive(Debug, Deserialize)]
pub struct Request {
    jsonrpc: String,
    pub(crate) method: String,
    pub(crate) params: Option<Vec<Value>>,
    pub(crate) id: Option<Value>,
}

impl Display for Request {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Request {{ jsonrpc: {}, method: {}, params: ..., id: {} }}",
            self.jsonrpc,
            self.method,
            to_string(&self.id).unwrap()
        )
    }
}

impl Request {
    pub fn parse(data: &[u8]) -> Result<Self, Option<Response>> {
        match serde_json::from_slice::<Self>(data) {
            Ok(req) if req.jsonrpc == "2.0" => Ok(req),
            Ok(req) if req.id.is_some() => {
                let err = Error::invalid_request(Some(format!("wrong version {}", req.jsonrpc)));
                Err(Some(Response::error(err, req.id)))
            }
            Ok(_) => Err(None),
            Err(err) => {
                let err = Error::parse_error(Some(format!("{}", err)));
                Err(Some(Response::error(err, None)))
            }
        }
    }
}
