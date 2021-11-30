//! JSON-RPC https://www.jsonrpc.org/specification
//!
//! 限制：不支持 batch request, 不支持命名参数

use std::future::Future;
use std::pin::Pin;

pub use futures;
use serde::Serialize;
use serde_json::{to_value, Value};
pub use ws;
pub use ws_jsonrpc_codegen::rpc;

use crate::response::Error;

pub mod handler;
mod request;
pub mod response;

#[macro_export]
macro_rules! method {
    ($name:ident) => {
        (stringify!($name), $name)
    };
}

pub type MethodReturnType = Pin<Box<dyn Future<Output = Result<Value, Error>> + Send>>;

pub type Method = fn(Vec<Value>) -> MethodReturnType;

#[inline]
pub fn convert<T, E>(result: Result<T, E>) -> Result<Value, Error>
where
    T: Serialize,
    Error: From<E>,
{
    Ok(to_value(result?).expect("serialize error"))
}
