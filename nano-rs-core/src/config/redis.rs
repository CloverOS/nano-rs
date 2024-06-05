use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RedisConfig {
    #[serde(flatten)]
    pub node: NodeConfig,
    pub cluster: Option<Vec<NodeConfig>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NodeConfig {
    pub tls: Option<bool>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub db: Option<u8>,
    #[serde(flatten)]
    pub redis_auth: Option<RedisAuth>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RedisAuth {
    #[serde(rename = "username")]
    pub user_name: Option<String>,
    pub password: String,
}

impl RedisConfig {
    pub fn get_cluster_url(&self) -> Vec<String> {
        if self.cluster.is_none() {
            vec![]
        } else {
            let mut urls = vec![];
            for x in self.cluster.clone().unwrap() {
                urls.push(x.get_node_url())
            }
            urls
        }
    }
}

impl NodeConfig {
    fn get_node_url(&self) -> String {
        let mut url = format!("{}:{}", self.host.clone().unwrap_or("127.0.0.1".to_string()), self.port.clone().unwrap_or(3306));
        if let Some(redis_auth) = self.redis_auth.clone() {
            url = format!("{}@{}", redis_auth.password, url);
            if let Some(user_name) = redis_auth.user_name.clone() {
                url = format!("{}:{}", user_name, url)
            }
        }
        if self.tls.unwrap_or(false) {
            format!("rediss://{}/{}", url, self.db.unwrap_or(0))
        } else {
            format!("redis://{}/{}", url, self.db.unwrap_or(0))
        }
    }
}