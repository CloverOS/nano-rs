use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct PrometheusConfig {
   pub enable: Option<bool>,
}