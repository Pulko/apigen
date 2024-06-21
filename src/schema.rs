use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ApiSchema {
    pub entities: Vec<Entity>,
}

#[derive(Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub field_type: String,
}
