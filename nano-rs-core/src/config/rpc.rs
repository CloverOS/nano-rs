use serde::{Deserialize, Serialize};

use crate::config::etcd::EtcdConfig;

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct RpcConfig {
    pub direct: Option<Vec<String>> ,
    pub balance: Option<bool>,
    pub key: Option<String>,
    pub etcd: Option<EtcdConfig>,
    pub timeout: Option<usize>,
}

impl RpcConfig {
    pub fn is_direct(&self) -> bool {
        if let Some(d) = &self.direct {
            return !d.is_empty();
        }
        false
    }
}