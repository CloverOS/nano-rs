use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Pet Data
#[derive(Deserialize, Serialize, Debug, ToSchema, IntoParams)]
pub struct Pet {
    /// The unique identifier for a pet
    pub id: i64,
    /// The name of the pet
    pub name: String,
    /// The tag of the pet
    pub tag: Option<String>,
    #[serde(flatten)]
    pub inline: Option<InlineThings>,
    /// The meta of the pet
    pub meta: Meta,
}

/// Pet Data
#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct PetForm {
    /// The unique identifier for a pet
    pub id: i64,
    /// The name of the pet
    pub name: String,
    /// The tag of the pet
    pub tag: Option<String>,
    #[serde(flatten)]
    pub inline: Option<InlineThings>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema, IntoParams)]
pub struct InlineThings {
    pub parent: i64,
}

#[derive(Deserialize, Serialize, Debug, ToSchema, IntoParams)]
pub struct Meta {
    ///  The name of the pet
    pub name: String,
    /// The age of the pet
    pub age: i64,
}

#[derive(Deserialize, IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
pub struct QueryPet {
    /// The unique identifier for a pet
    pub id: i64,
}

#[derive(Debug, Deserialize, Serialize, IntoParams)]
pub struct Params {
    /// The unique identifier for a pet
    pub id: i64,
}

#[derive(Debug, Deserialize, Serialize, IntoParams)]
pub struct Page {
    /// page start 1
    pub page: i64,
    /// count ,default 10
    pub count: i64,
}