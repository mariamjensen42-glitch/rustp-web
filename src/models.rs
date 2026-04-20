use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub slug: Option<String>,
    pub content: String,
    pub excerpt: Option<String>,
    pub author: String,
    pub status: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub published_at: Option<chrono::NaiveDateTime>,
    pub deleted_at: Option<chrono::NaiveDateTime>,
    pub category_id: Option<i32>,
    pub user_id: Option<i32>,
    pub summary: Option<String>,
    pub cover_image: Option<String>,
    pub is_published: Option<bool>,
    pub is_top: Option<bool>,
    pub allow_comments: Option<bool>,
    pub view_count: Option<i32>,
    pub scheduled_at: Option<chrono::NaiveDateTime>,
    pub is_scheduled: Option<bool>,
    pub draft_saved_at: Option<chrono::NaiveDateTime>,
    pub auto_save_draft: Option<bool>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::posts)]
pub struct NewPost {
    pub title: String,
    pub slug: Option<String>,
    pub content: String,
    pub excerpt: Option<String>,
    pub author: String,
    pub status: Option<String>,
    pub category_id: Option<i32>,
    pub user_id: Option<i32>,
    pub summary: Option<String>,
    pub cover_image: Option<String>,
    pub is_published: Option<bool>,
    pub is_top: Option<bool>,
    pub allow_comments: Option<bool>,
    pub scheduled_at: Option<chrono::NaiveDateTime>,
    pub is_scheduled: Option<bool>,
    pub auto_save_draft: Option<bool>,
}

#[derive(AsChangeset, Deserialize, Default)]
#[diesel(table_name = crate::schema::posts)]
pub struct UpdatePost {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub excerpt: Option<String>,
    pub author: Option<String>,
    pub status: Option<String>,
    pub published_at: Option<chrono::NaiveDateTime>,
    pub deleted_at: Option<chrono::NaiveDateTime>,
    pub category_id: Option<i32>,
    pub user_id: Option<i32>,
    pub summary: Option<String>,
    pub cover_image: Option<String>,
    pub is_published: Option<bool>,
    pub is_top: Option<bool>,
    pub allow_comments: Option<bool>,
    pub view_count: Option<i32>,
    pub scheduled_at: Option<chrono::NaiveDateTime>,
    pub is_scheduled: Option<bool>,
    pub draft_saved_at: Option<chrono::NaiveDateTime>,
    pub auto_save_draft: Option<bool>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct PostVersion {
    pub id: i32,
    pub post_id: i32,
    pub version_number: i32,
    pub title: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub summary: Option<String>,
    pub cover_image: Option<String>,
    pub created_by: Option<i32>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::post_versions)]
pub struct NewPostVersion {
    pub post_id: i32,
    pub version_number: i32,
    pub title: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub summary: Option<String>,
    pub cover_image: Option<String>,
    pub created_by: Option<i32>,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct PostAnalytic {
    pub id: i32,
    pub post_id: i32,
    pub visit_date: chrono::NaiveDate,
    pub visit_count: i32,
    pub unique_visitors: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::post_analytics)]
pub struct NewPostAnalytic {
    pub post_id: i32,
    pub visit_date: chrono::NaiveDate,
    pub visit_count: i32,
    pub unique_visitors: i32,
}

#[derive(AsChangeset, Deserialize, Default)]
#[diesel(table_name = crate::schema::post_analytics)]
pub struct UpdatePostAnalytic {
    pub visit_count: Option<i32>,
    pub unique_visitors: Option<i32>,
    pub updated_at: Option<chrono::NaiveDateTime>,
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
    pub slug: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::categories)]
pub struct NewCategory {
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::categories)]
pub struct UpdateCategory {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub slug: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::tags)]
pub struct NewTag {
    pub name: String,
    pub slug: Option<String>,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::tags)]
pub struct UpdateTag {
    pub name: Option<String>,
    pub slug: Option<String>,
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

#[derive(Queryable, Serialize, Deserialize)]
pub struct Media {
    pub id: i32,
    pub filename: String,
    pub filepath: String,
    pub mimetype: String,
    pub size: i64,
    pub uploaded_by: Option<i32>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::media)]
pub struct NewMedia {
    pub filename: String,
    pub filepath: String,
    pub mimetype: String,
    pub size: i64,
    pub uploaded_by: Option<i32>,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct UserReadHistory {
    pub id: i32,
    pub user_id: i32,
    pub post_id: i32,
    pub read_at: chrono::NaiveDateTime,
    pub read_duration: Option<i32>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::user_read_history)]
pub struct NewUserReadHistory {
    pub user_id: i32,
    pub post_id: i32,
    pub read_duration: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct RecommendedPost {
    pub id: i32,
    pub title: String,
    pub slug: Option<String>,
    pub excerpt: Option<String>,
    pub cover_image: Option<String>,
    pub author: String,
    pub published_at: Option<chrono::NaiveDateTime>,
    pub category_name: Option<String>,
    pub tag_names: Vec<String>,
    pub view_count: Option<i32>,
    pub relevance_score: i32,
}

#[derive(Serialize, Deserialize)]
pub struct PostWithRecommendations {
    pub post: Post,
    pub recommendations: Option<Vec<RecommendedPost>>,
}

#[derive(Serialize, Deserialize)]
pub struct ReadHistoryWithPost {
    pub id: i32,
    pub read_at: chrono::NaiveDateTime,
    pub read_duration: Option<i32>,
    pub post: Post,
}
