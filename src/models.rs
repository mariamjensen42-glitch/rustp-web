use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub author: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::posts)]
pub struct NewPost {
    pub title: String,
    pub content: String,
    pub author: String,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::posts)]
pub struct UpdatePost {
    pub title: Option<String>,
    pub content: Option<String>,
    pub author: Option<String>,
}
