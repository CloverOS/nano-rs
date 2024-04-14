use std::fs;
use std::io::Error;
use serde::Deserialize;

use crate::config::rest::RestConfig;

/// 读取服务配置
/// Read rest service configuration
pub fn read_rest_config(path: &str) -> Result<RestConfig, Error> {
    let content = fs::read_to_string(path)?;
    let service = RestConfig::from_str(content.as_str()).expect("read_service_config failed");
    Ok(service)
}

/// 读取配置
/// Read configuration
pub fn read_config<T>(path: &str) -> Result<T, Error> where T: for<'a> Deserialize<'a> {
    let content = fs::read_to_string(path)?;
    let config = serde_yaml::from_str(content.as_str()).expect("read config failed");
    Ok(config)
}