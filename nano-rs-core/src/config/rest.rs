use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::config::logger::LogConfig;
use crate::config::prometheus::PrometheusConfig;
use crate::config::rpc::RpcConfig;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct RestConfig {
    /// server port
    pub port: u16,
    /// server name (for micro-server reserved)
    pub name: String,
    /// server listening address
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// server mode dev,prod,test etc..
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// server connect timeout (second)
    pub time_out: Option<usize>,
    /// server base route path, "/v1" "/v2"
    pub base_path: Option<String>,
    /// log detail config
    #[serde(default)]
    pub log: LogConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// rpc server config
    pub rpc: Option<HashMap<String, RpcConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// prometheus config
    pub prometheus: Option<PrometheusConfig>,
}

impl RestConfig {
    pub fn from_str(s: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(s)
    }

    pub fn get_env_filter(&self) -> String {
        let mut filter = String::new();
        //默认配置
        filter.push_str(&format!("{}=debug,", &self.name));
        filter.push_str(&format!("{}=debug,", "nano_rs_extra"));
        filter.push_str(&format!("{}=debug,", "hyper"));
        for (k, v) in self.log.logging.iter() {
            filter.push_str(&format!("{}={},", k, v.clone().level.unwrap_or("debug".to_string())));
        }
        if !filter.is_empty() {
            filter.pop();
        }
        filter
    }

    pub fn get_rpc_config(&self, key: &str) -> Result<RpcConfig, String> {
        if let Some(rpc) = &self.rpc {
            if let Some(rpc_config) = rpc.get(key) {
                let mut config = rpc_config.clone();
                config.key = Some(key.to_string());
                return Ok(config);
            }
        }
        Err(String::from("rpc config not found"))
    }
}