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
    pub category_id: Option<i32>,
    pub user_id: Option<i32>,
    pub summary: Option<String>,
    pub cover_image: Option<String>,
    pub is_published: Option<bool>,
    pub is_top: Option<bool>,
    pub allow_comments: Option<bool>,
    pub view_count: Option<i32>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::posts)]
pub struct NewPost {
    pub title: String,
    pub content: String,
    pub author: String,
    pub category_id: Option<i32>,
    pub user_id: Option<i32>,
    pub summary: Option<String>,
    pub cover_image: Option<String>,
    pub is_published: bool,
    pub is_top: bool,
    pub allow_comments: bool,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::posts)]
pub struct UpdatePost {
    pub title: Option<String>,
    pub content: Option<String>,
    pub author: Option<String>,
    pub category_id: Option<i32>,
    pub user_id: Option<i32>,
    pub summary: Option<String>,
    pub cover_image: Option<String>,
    pub is_published: Option<bool>,
    pub is_top: Option<bool>,
    pub allow_comments: Option<bool>,
    pub view_count: Option<i32>,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub avatar: Option<String>,
    pub bio: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub avatar: Option<String>,
    pub bio: Option<String>,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::users)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub role: Option<String>,
    pub avatar: Option<String>,
    pub bio: Option<String>,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub parent_id: Option<i32>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::categories)]
pub struct NewCategory {
    pub name: String,
    pub parent_id: Option<i32>,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::categories)]
pub struct UpdateCategory {
    pub name: Option<String>,
    pub parent_id: Option<i32>,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::tags)]
pub struct NewTag {
    pub name: String,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::tags)]
pub struct UpdateTag {
    pub name: Option<String>,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct Comment {
    pub id: i32,
    pub post_id: i32,
    pub user_id: Option<i32>,
    pub parent_id: Option<i32>,
    pub content: String,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub author_website: Option<String>,
    pub status: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::comments)]
pub struct NewComment {
    pub post_id: i32,
    pub user_id: Option<i32>,
    pub parent_id: Option<i32>,
    pub content: String,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub author_website: Option<String>,
    pub status: String,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::comments)]
pub struct UpdateComment {
    pub content: Option<String>,
    pub status: Option<String>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::post_tags)]
pub struct PostTag {
    pub post_id: i32,
    pub tag_id: i32,
}
