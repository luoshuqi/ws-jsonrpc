use std::convert::Infallible;

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct Response {
    jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Error>,
    id: Option<Value>,
}

impl Response {
    pub fn ok(result: Value, id: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            result: Some(result),
            error: None,
            id: Some(id),
        }
    }

    pub fn error(error: Error, id: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0",
            result: None,
            error: Some(error),
            id,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Error {
    code: i32,
    message: String,
    data: Option<Value>,
}

impl From<Infallible> for Error {
    fn from(v: Infallible) -> Self {
        match v {}
    }
}

impl Error {
    pub fn new<T: Serialize>(code: i32, message: impl ToString, data: Option<T>) -> Self {
        Self {
            code,
            message: message.to_string(),
            data: data.map(|v| serde_json::to_value(v).unwrap()),
        }
    }

    pub fn parse_error<T: Serialize>(data: Option<T>) -> Self {
        Self::new(-32700, "Parse error", data)
    }

    pub fn invalid_request<T: Serialize>(data: Option<T>) -> Self {
        Self::new(-32600, "Invalid Request", data)
    }

    pub fn method_not_found<T: Serialize>(data: Option<T>) -> Self {
        Self::new(-32601, "Method not found", data)
    }

    pub fn invalid_params<T: Serialize>(data: Option<T>) -> Self {
        Self::new(-32602, "Invalid params", data)
    }

    pub fn internal_error<T: Serialize>(data: Option<T>) -> Self {
        Self::new(-32603, "Internal error", data)
    }

    pub fn server_error<T: Serialize>(
        code: Option<i32>,
        message: impl ToString,
        data: Option<T>,
    ) -> Self {
        let code = code.unwrap_or(-32000);
        debug_assert!(code <= -32000 && code >= -32099);
        Self::new(code, message, data)
    }
}
