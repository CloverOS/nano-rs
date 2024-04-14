use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct EtcdConfig {
    pub host: Option<Vec<String>>,
    pub protocol: Option<String>,
    pub user: Option<String>,
    pub pass_word: Option<String>,
}