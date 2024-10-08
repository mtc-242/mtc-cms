use serde::{Deserialize, Serialize};
use surrealdb_sql::Datetime;
use validator::Validate;

use crate::from_thing;

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq)]
pub struct GroupModel {
    #[serde(deserialize_with = "from_thing")]
    pub id: String,
    pub slug: String,
    pub title: String,
    pub created_at: Datetime,
    pub updated_at: Datetime,
    pub created_by: String,
    pub updated_by: String,
}

impl Default for GroupModel {
    fn default() -> Self {
        Self {
            id: "".to_string(),
            slug: "".to_string(),
            title: "".to_string(),
            created_at: Default::default(),
            updated_at: Default::default(),
            created_by: "".to_string(),
            updated_by: "".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Validate)]
pub struct GroupCreateModel {
    pub title: String,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct GroupUpdateModel {
    pub title: String,
}
