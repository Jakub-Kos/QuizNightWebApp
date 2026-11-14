//! Browser Google token, Drive and private Sheets integration. See README.md for UI wiring.
// Step two supplies UI consumers; keep the complete non-UI API available meanwhile.
#![allow(dead_code)]
pub mod auth;
pub mod drive;
pub mod fetch;
pub mod sheet_edit;
pub mod sheets;
pub mod sync;
pub use quiz_core::quiz::drive as config;
use serde_json::Value;
use wasm_bindgen::JsValue;
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Error {
    Auth(String),
    Api(u16, String),
    Blocked(String),
    Drive(String, String),
    Sheet(String),
    Other(String),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Auth(s) | Self::Blocked(s) | Self::Sheet(s) | Self::Other(s) => s,
                Self::Api(_, s) | Self::Drive(_, s) => s,
            }
        )
    }
}
impl std::error::Error for Error {}
impl From<JsValue> for Error {
    fn from(v: JsValue) -> Self {
        Self::Other(v.as_string().unwrap_or_else(|| format!("{v:?}")))
    }
}
impl From<quiz_store::Error> for Error {
    fn from(v: quiz_store::Error) -> Self {
        Self::Other(v.to_string())
    }
}
pub type Result<T> = std::result::Result<T, Error>;
pub(super) fn js(v: &Value) -> JsValue {
    js_sys::JSON::parse(&v.to_string()).unwrap()
}
pub(super) fn value(v: JsValue) -> Result<Value> {
    let s = js_sys::JSON::stringify(&v)
        .map_err(Error::from)?
        .as_string()
        .unwrap_or_default();
    serde_json::from_str(&s).map_err(|e| Error::Other(e.to_string()))
}
pub(super) fn string(v: &Value, key: &str) -> String {
    v[key].as_str().unwrap_or_default().into()
}
pub(super) fn enc(s: &str) -> String {
    config::encode_component(s)
}
pub(super) fn now() -> f64 {
    js_sys::Date::now()
}
