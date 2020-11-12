use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

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
    result: serde_json::Value,
}

impl ActivationResponse {
    pub fn new(status: ActivationResponseStatus, result: serde_json::Value) -> Self {
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
    pub value: ActivationInitInner,
}

#[derive(Debug, Deserialize)]
pub struct ActivationInitInner {
    pub name: String,
    pub main: String,
    pub code: String,
    pub binary: bool,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct ActivationContext {
    pub value: serde_json::Value,
    pub namespace: String,
    pub action_name: String,
    pub api_host: Option<String>,
    pub api_key: Option<String>,
    pub activation_id: String,
    pub transaction_id: String,
    #[serde(deserialize_with = "str_to_u64")]
    pub deadline: u64,
}

fn str_to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    buf.parse::<u64>().map_err(serde::de::Error::custom)
}
