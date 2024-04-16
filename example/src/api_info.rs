#[allow(dead_code)]
pub fn get_api_info() -> Vec<ApiInfo> {
    vec![
        ApiInfo { method : "post".to_string(), path : "/samoyed/shower".to_string(),
        base_path : "".to_string(), handler_fun : "shower".to_string(), summary :
        "Give your Samoyed a bath".to_string(), public : false, group_name : "Samoyed"
        .to_string(), }, ApiInfo { method : "get".to_string(), path : "/store/name"
        .to_string(), base_path : "".to_string(), handler_fun : "get_store_name"
        .to_string(), summary : "Get the default pet store name".to_string(), public :
        false, group_name : "Store".to_string(), }, ApiInfo { method : "get".to_string(),
        path : "/samoyed/name".to_string(), base_path : "".to_string(), handler_fun :
        "name".to_string(), summary : "Get Samoyed name".to_string(), public : false,
        group_name : "Samoyed".to_string(), }, ApiInfo { method : "get".to_string(), path
        : "/store/tel".to_string(), base_path : "".to_string(), handler_fun :
        "get_store_tel".to_string(), summary : "Get Store's telephone number"
        .to_string(), public : false, group_name : "Store".to_string(), }, ApiInfo {
        method : "get".to_string(), path : "/samoyed/miss".to_string(), base_path : ""
        .to_string(), handler_fun : "miss".to_string(), summary : "Miss mantou so much"
        .to_string(), public : false, group_name : "Samoyed".to_string(), }, ApiInfo {
        method : "get".to_string(), path : "/samoyed/:name".to_string(), base_path : ""
        .to_string(), handler_fun : "hello".to_string(), summary : "Say Hello to name"
        .to_string(), public : false, group_name : "Samoyed".to_string(), }
    ]
}
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ApiInfo {
    pub method: String,
    pub path: String,
    pub base_path: String,
    pub handler_fun: String,
    pub summary: String,
    pub public: bool,
    pub group_name: String,
}
