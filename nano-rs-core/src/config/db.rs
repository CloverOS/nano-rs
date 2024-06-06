use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct DataBaseConfig {
    pub port: u16,
    pub username: String,
    pub password: String,
    pub host: String,
    pub database: String,
    #[serde(default = "default_sqlx_logging")]
    pub sqlx_logging: bool,
    #[serde(default = "default_logging_level")]
    pub logging_level: isize,
    #[serde(default = "default_config")]
    pub config: String,
    /// 空闲中的最大连接数
    #[serde(default = "default_max_idle_conns")]
    pub max_idle_conns: u32,
    /// 打开到数据库的最大连接数
    #[serde(default = "default_max_open_conns")]
    pub max_open_conns: u32,
}

fn default_sqlx_logging() -> bool {
    true
}

fn default_logging_level() -> isize {
    2
}

fn default_config() -> String {
    "".to_string()
}

fn default_max_idle_conns() -> u32 {
    100
}

fn default_max_open_conns() -> u32 {
    100
}