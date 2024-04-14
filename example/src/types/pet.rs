use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct PetShower {
    pub name: String,
    pub status: String,
}