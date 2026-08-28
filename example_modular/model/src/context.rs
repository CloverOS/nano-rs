use nano_rs::config::rest::RestConfig;

#[derive(Clone)]
pub struct ServiceContext {
    pub rest_config: RestConfig,
}
