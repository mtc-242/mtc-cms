use serde::{Deserialize, Serialize};
use surrealdb::sql::Datetime;
use validator::Validate;

use crate::model::from_thing;

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct RoleModel {
    #[serde(deserialize_with = "from_thing")]
    pub id: String,
    pub name: String,
    pub title: String,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

#[derive(Deserialize, Validate)]
pub struct RoleCreateModel {
    pub name: String,
    pub title: String,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct RoleUpdateModel {
    pub name: String,
    pub title: String,
}
