use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::default::Default;

#[derive(PartialEq, Clone)]
pub enum ActivationResponseStatus {
    Success = 0,
    // ApplicationError = 1,
    // ActionDeveloperError = 2,
    // WhiskInternalError = 3,
}

impl Serialize for ActivationResponseStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use ActivationResponseStatus::*;

        match self {
            Success => serializer.serialize_str("success"),
            // ApplicationError => serializer.serialize_str("application error"),
            // ActionDeveloperError => serializer.serialize_str("action developer error"),
            // WhiskInternalError => serializer.serialize_str("whisk internal error"),
        }
    }
}

#[derive(Serialize)]
pub struct ActivationResponse {
    status: ActivationResponseStatus,
    status_code: u8,
    success: bool,
    result: HashMap<String, String>,
}

impl Default for ActivationResponse {
    fn default() -> Self {
        Self::new(ActivationResponseStatus::Success, HashMap::new())
    }
}

impl ActivationResponse {
    fn new(status: ActivationResponseStatus, result: HashMap<String, String>) -> Self {
        Self {
            success: status == ActivationResponseStatus::Success,
            status_code: status.clone() as u8,
            status,
            result,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ActivationInit {
    value: ActivationInitInner,
}

#[derive(Debug, Deserialize)]
pub struct ActivationInitInner {
    name: String,
    main: String,
    code: String,
    binary: bool,
    env: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct ActivationContext {
    value: serde_json::Value,
    namespace: String,
    action_name: String,
    api_host: Option<String>,
    api_key: Option<String>,
    activation_id: String,
    transaction_id: String,
    #[serde(deserialize_with = "str_to_u64")]
    deadline: u64,
}

fn str_to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    buf.parse::<u64>().map_err(serde::de::Error::custom)
}
