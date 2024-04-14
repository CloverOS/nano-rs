use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiFnDoc {
    ///接口名称
    pub api: String,
    ///接口组
    pub group: String,
}