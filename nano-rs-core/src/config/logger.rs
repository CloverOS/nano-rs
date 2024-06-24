use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct LogConfig {
    /// enable log with request,default true
    pub log_req: Option<bool>,
    /// enable log request body,default false
    pub enable_request_body_log: Option<bool>,
    /// tracing env filter
    #[serde(default = "default_logging")]
    pub logging: HashMap<String, LogLevel>,
    /// level config
    pub level: Option<Level>,
    /// ansi
    pub ansi: Option<bool>,
}

fn default_logging() -> HashMap<String, LogLevel> {
    let mut map = HashMap::new();
    map.insert("tower_http".to_string(), LogLevel::default());
    map
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct LogLevel {
    pub level: Option<String>,
}

/// log file config
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct LogFileConfig {
    /// file dir
    pub dir: Option<String>,
    /// file prefix name
    pub prefix: Option<String>,
    /// enable file output
    pub file: Option<bool>,
}

impl LogFileConfig {
    pub fn get_default_prefix(&self, level: &str) -> &str {
        match level {
            "trace" => {
                "trace.log"
            }
            "debug" => {
                "debug.log"
            }
            "info" => {
                "info.log"
            }
            "warn" => {
                "warn.log"
            }
            "error" => {
                "error.log"
            }
            &_ => {
                "info.log"
            }
        }
    }

    pub fn get_tracing_level(&self, level: &str) -> tracing::Level {
        match level {
            "trace" => {
                tracing::Level::TRACE
            }
            "debug" => {
                tracing::Level::DEBUG
            }
            "info" => {
                tracing::Level::INFO
            }
            "warn" => {
                tracing::Level::WARN
            }
            "error" => {
                tracing::Level::ERROR
            }
            &_ => {
                tracing::Level::INFO
            }
        }
    }
}

/// level config
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct Level {
    /// trace file config
    pub trace: Option<LogFileConfig>,
    /// debug file config
    pub debug: Option<LogFileConfig>,
    /// info file config
    pub info: Option<LogFileConfig>,
    /// warn file config
    pub warn: Option<LogFileConfig>,
    /// error file config
    pub error: Option<LogFileConfig>,
}

impl Level {
    pub fn get_log_level(&self) -> String {
        if self.trace.is_some() {
            "trace".to_string()
        } else if self.debug.is_some() {
            "debug".to_string()
        } else if self.info.is_some() {
            "info".to_string()
        } else if self.warn.is_some() {
            "warn".to_string()
        } else if self.error.is_some() {
            "error".to_string()
        } else {
            "info".to_string()
        }
    }

    pub fn get_log_file_config(&self, level: &str) -> Option<LogFileConfig> {
        match level {
            "trace" => {
                self.trace.clone()
            }
            "debug" => {
                self.debug.clone()
            }
            "info" => {
                self.info.clone()
            }
            "warn" => {
                self.warn.clone()
            }
            "error" => {
                self.error.clone()
            }
            &_ => {
                None
            }
        }
    }

    pub fn get_tracing_level(&self) -> tracing::Level {
        if self.trace.is_some() {
            tracing::Level::TRACE
        } else if self.debug.is_some() {
            tracing::Level::DEBUG
        } else if self.info.is_some() {
            tracing::Level::INFO
        } else if self.warn.is_some() {
            tracing::Level::WARN
        } else if self.error.is_some() {
            tracing::Level::ERROR
        } else {
            tracing::Level::DEBUG
        }
    }
}

pub const LOG_LEVEL: [&str; 5] = ["trace", "debug", "info", "warn", "error"];