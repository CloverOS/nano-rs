use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct LoggerConfig {
    #[serde(default = "default_logging")]
    pub logging: HashMap<String, LoggerFileConfig>,

    #[serde(default = "default_enable_request_body_log")]
    pub enable_request_body_log: bool,
}

fn default_enable_request_body_log() -> bool {
    true
}

fn default_logging() -> HashMap<String, LoggerFileConfig> {
    let mut map = HashMap::new();
    map.insert("tower_http".to_string(), LoggerFileConfig {
        name: None,
        level: "debug".to_string(),
        dir: None,
        debug: None,
        info: None,
        trace: None,
        error: None,
        warn: None,
    });
    map
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct LoggerFileConfig {
    pub name: Option<String>,
    #[serde(default = "default_level")]
    pub level: String,
    pub dir: Option<String>,
    pub debug: Option<String>,
    pub info: Option<String>,
    pub trace: Option<String>,
    pub error: Option<String>,
    pub warn: Option<String>,
}

fn default_level() -> String {
    "info".to_string()
}