use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PrometheusConfig {
   pub enable: Option<bool>,
}

impl Default for PrometheusConfig{
   fn default() -> Self {
      PrometheusConfig{
         enable: Some(true),
      }
   }
}