use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct PageData<T>
where
    T: ToSchema,
{
    pub total: u64,
    pub page: u64,
    pub count: u64,
    pub data: Vec<T>,
}
