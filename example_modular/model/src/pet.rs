use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct Pet {
    pub id: i64,
    pub name: String,
    pub tag: Option<String>,
    pub meta: Meta,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct PetForm {
    pub id: i64,
    pub name: String,
    pub tag: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct PetShower {
    pub name: String,
    pub status: String,
}

#[derive(Deserialize, Serialize, Debug, ToSchema, IntoParams)]
pub struct Meta {
    pub age: i64,
}

#[derive(Deserialize, Serialize, Debug, IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
pub struct QueryPet {
    pub id: i64,
}

#[derive(Debug, Deserialize, Serialize, IntoParams)]
pub struct Params {
    pub id: i64,
}

#[derive(Debug, Deserialize, Serialize, IntoParams)]
pub struct Page {
    pub page: i64,
    pub count: i64,
}
