#[allow(dead_code)]
pub fn get_api_info() -> Vec<ApiInfo> {
    vec![
        ApiInfo { method : "get".to_string(), path : "/samoyed/:name".to_string(),
        base_path : "".to_string(), handler_fun : "hello".to_string(), summary :
        " Say Hello to name".to_string(), public : false, group_name : "Samoyed"
        .to_string(), }, ApiInfo { method : "get".to_string(), path : "/samoyed/miss"
        .to_string(), base_path : "".to_string(), handler_fun : "miss".to_string(),
        summary : " Miss mantou so much".to_string(), public : false, group_name :
        "Samoyed".to_string(), }, ApiInfo { method : "get".to_string(), path :
        "/samoyed/name".to_string(), base_path : "".to_string(), handler_fun : "name"
        .to_string(), summary : " Get Samoyed name".to_string(), public : false,
        group_name : "Samoyed".to_string(), }, ApiInfo { method : "post".to_string(),
        path : "/samoyed/shower".to_string(), base_path : "".to_string(), handler_fun :
        "shower".to_string(), summary : " Give your Samoyed a bath".to_string(), public :
        false, group_name : "Samoyed".to_string(), }, ApiInfo { method : "post"
        .to_string(), path : "/store/pet/form".to_string(), base_path : "".to_string(),
        handler_fun : "add_form_pet".to_string(), summary : " Add a new pet to the store"
        .to_string(), public : false, group_name : "Store".to_string(), }, ApiInfo {
        method : "post".to_string(), path : "/store/pet/json".to_string(), base_path : ""
        .to_string(), handler_fun : "add_json_pet".to_string(), summary :
        " Add a new pet to the store".to_string(), public : false, group_name : "Store"
        .to_string(), }, ApiInfo { method : "get".to_string(), path : "/store/pet/:id"
        .to_string(), base_path : "".to_string(), handler_fun : "get_pet_name"
        .to_string(), summary : " Get pet by id".to_string(), public : false, group_name
        : "Store".to_string(), }, ApiInfo { method : "get".to_string(), path :
        "/store/pet/list/:page/:count/:id".to_string(), base_path : "".to_string(),
        handler_fun : "get_pet_name_list".to_string(), summary : " Get pet list"
        .to_string(), public : false, group_name : "Store".to_string(), }, ApiInfo {
        method : "get".to_string(), path : "/store/pet".to_string(), base_path : ""
        .to_string(), handler_fun : "get_query_pet_name".to_string(), summary :
        " Get pet by id".to_string(), public : false, group_name : "Store".to_string(),
        }, ApiInfo { method : "get".to_string(), path : "/store/name".to_string(),
        base_path : "".to_string(), handler_fun : "get_store_name".to_string(), summary :
        " Get the default pet store name".to_string(), public : false, group_name :
        "Store".to_string(), }, ApiInfo { method : "get".to_string(), path : "/store/tel"
        .to_string(), base_path : "".to_string(), handler_fun : "get_store_tel"
        .to_string(), summary : " Get Store's telephone number".to_string(), public :
        false, group_name : "Store".to_string(), }, ApiInfo { method : "post"
        .to_string(), path : "/store/pet/list/:page/:count".to_string(), base_path : ""
        .to_string(), handler_fun : "pet_page_list".to_string(), summary :
        " Add a new pet to the store".to_string(), public : false, group_name : "Store"
        .to_string(), }
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
