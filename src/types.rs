use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LogLevel {
    INFO,
    WARNING,
    ERROR,
    DEBUG,
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub key: String,
    pub value: String,
}

impl Attribute {
    pub fn new<K: Into<String>, V: Into<String>>(key: K, value: V) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LogMessage {
    pub timestamp: String,
    pub body: String,
    pub level: LogLevel,
    pub attributes: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Logs,
}

#[derive(Debug, Serialize)]
pub struct MessageBatch {
    pub token: String,
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub logs: Vec<LogMessage>,
}
