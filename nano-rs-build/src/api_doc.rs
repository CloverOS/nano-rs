use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiFnDoc {
    ///接口名称
    /// Api Name
    pub api: String,
    ///接口描述
    /// Api Description
    pub api_desc: String,
    ///接口分组
    /// Api Group
    pub api_group: String,
}