use actix_web::{web, HttpResponse, Responder, HttpRequest, HttpMessage};
use actix_multipart::Multipart;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use futures_util::stream::StreamExt;
use futures_util::stream::TryStreamExt;
use crate::{models::{Post, NewPost, UpdatePost, User, NewUser, Comment, NewComment, UpdateComment, Category, NewCategory, UpdateCategory, Tag, NewTag, UpdateTag, PostTag, Media, NewMedia, PostVersion, NewPostVersion}, schema::posts, schema::users, schema::comments, schema::categories, schema::tags, schema::post_tags, schema::media, schema::post_versions, db::establish_connection, auth::{generate_token, hash_password, verify_password, verify_token, Claims}};
use validator::Validate;
use chrono::Utc;
use std::sync::Mutex;
use std::collections::HashSet;

pub struct AppState {
    pub blacklist: Mutex<HashSet<String>>,
}

fn extract_token(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

fn get_current_user(req: &HttpRequest) -> Option<Claims> {
    let token = extract_token(req)?;
    verify_token(&token).ok()
}

fn is_admin(req: &HttpRequest) -> bool {
    get_current_user(req).map(|c| c.role == "admin").unwrap_or(false)
}

fn get_user_from_request(req: &HttpRequest) -> Option<Claims> {
    get_current_user(req)
}

#[derive(serde::Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub sort_by: Option<String>,
    pub order: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct PostFilterQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub sort_by: Option<String>,
    pub order: Option<String>,
    pub category_id: Option<i32>,
    pub tag_id: Option<i32>,
    pub q: Option<String>,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
    pub total_pages: i64,
}

#[derive(serde::Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(serde::Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(serde::Deserialize)]
pub struct UpdatePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(serde::Deserialize)]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
    pub bio: Option<String>,
    pub avatar: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub slug: Option<String>,
    pub content: String,
    pub excerpt: Option<String>,
    pub status: Option<String>,
    pub category_id: Option<i32>,
    pub summary: Option<String>,
    pub cover_image: Option<String>,
    pub is_published: Option<bool>,
    pub is_top: Option<bool>,
    pub allow_comments: Option<bool>,
    pub published_at: Option<chrono::NaiveDateTime>,
    pub tag_ids: Option<Vec<i32>>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub seo_keywords: Option<String>,
    pub seo_canonical: Option<String>,
    pub seo_robots: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub excerpt: Option<String>,
    pub status: Option<String>,
    pub category_id: Option<i32>,
    pub summary: Option<String>,
    pub cover_image: Option<String>,
    pub is_published: Option<bool>,
    pub is_top: Option<bool>,
    pub allow_comments: Option<bool>,
    pub published_at: Option<chrono::NaiveDateTime>,
    pub tag_ids: Option<Vec<i32>>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub seo_keywords: Option<String>,
    pub seo_canonical: Option<String>,
    pub seo_robots: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateStatusRequest {
    pub status: String,
}

#[derive(serde::Deserialize)]
pub struct CommentRequest {
    pub post_id: i32,
    pub content: String,
    pub parent_id: Option<i32>,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub author_website: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct CategoryRequest {
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
}

#[derive(serde::Deserialize)]
pub struct TagRequest {
    pub name: String,
    pub slug: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(serde::Deserialize)]
pub struct UserFilterQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub sort_by: Option<String>,
    pub order: Option<String>,
    pub q: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub bio: Option<String>,
    pub avatar: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct PostTagsRequest {
    pub post_id: i32,
    pub tag_ids: Vec<i32>,
}

#[derive(serde::Deserialize)]
pub struct CommentFilterQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub status: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct BatchCommentRequest {
    pub comment_ids: Vec<i32>,
}

pub async fn register(req: web::Json<RegisterRequest>) -> impl Responder {
    if let Err(errors) = req.validate() {
        return HttpResponse::BadRequest().json(errors);
    }

    let mut conn = establish_connection();

    if users::table.filter(users::email.eq(&req.email)).first::<User>(&mut conn).is_ok() {
        return HttpResponse::BadRequest().json("Email already exists");
    }

    if users::table.filter(users::username.eq(&req.username)).first::<User>(&mut conn).is_ok() {
        return HttpResponse::BadRequest().json("Username already exists");
    }

    let password_hash = match hash_password(&req.password) {
        Ok(h) => h,
        Err(e) => return HttpResponse::InternalServerError().json(format!("Password error: {}", e)),
    };

    let new_user = NewUser {
        username: req.username.clone(),
        email: req.email.clone(),
        password_hash,
        role: "user".to_string(),
        avatar: None,
        bio: None,
    };

    match diesel::insert_into(users::table)
        .values(&new_user)
        .execute(&mut conn)
    {
        Ok(_) => HttpResponse::Created().json(serde_json::json!({"message": "User registered successfully"})),
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
    }
}

pub async fn login(req: web::Json<LoginRequest>) -> impl Responder {
    if let Err(errors) = req.validate() {
        return HttpResponse::BadRequest().json(errors);
    }

    let mut conn = establish_connection();

    match users::table.filter(users::email.eq(&req.email)).first::<User>(&mut conn) {
        Ok(user) => {
            if verify_password(&req.password, &user.password_hash).unwrap_or(false) {
                let token = match generate_token(user.id, &user.username, &user.role) {
                    Ok(t) => t,
                    Err(e) => return HttpResponse::InternalServerError().json(format!("Token error: {}", e)),
                };
                HttpResponse::Ok().json(serde_json::json!({
                    "token": token,
                    "user": {
                        "id": user.id,
                        "username": user.username,
                        "email": user.email,
                        "role": user.role
                    }
                }))
            } else {
                HttpResponse::Unauthorized().json("Invalid password")
            }
        },
        Err(_) => HttpResponse::Unauthorized().json("User not found"),
    }
}

pub async fn logout(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    if let Some(token) = extract_token(&req) {
        let mut blacklist = state.blacklist.lock().unwrap();
        blacklist.insert(token);
    }
    HttpResponse::Ok().json(serde_json::json!({"message": "Logged out successfully"}))
}

pub async fn refresh_token(req: web::Json<RefreshRequest>) -> impl Responder {
    match verify_token(&req.refresh_token) {
        Ok(claims) => {
            let new_token = match generate_token(claims.user_id, &claims.username, &claims.role) {
                Ok(t) => t,
                Err(e) => return HttpResponse::InternalServerError().json(format!("Token error: {}", e)),
            };
            HttpResponse::Ok().json(serde_json::json!({"token": new_token}))
        },
        Err(e) => HttpResponse::Unauthorized().json(format!("Invalid token: {}", e)),
    }
}

pub async fn get_me(req: HttpRequest) -> impl Responder {
    match get_current_user(&req) {
        Some(claims) => {
            let mut conn = establish_connection();
            match users::table.find(claims.user_id).first::<User>(&mut conn) {
                Ok(user) => HttpResponse::Ok().json(serde_json::json!({
                    "id": user.id,
                    "username": user.username,
                    "email": user.email,
                    "role": user.role,
                    "avatar": user.avatar,
                    "bio": user.bio
                })),
                Err(_) => HttpResponse::NotFound().json("User not found"),
            }
        },
        None => HttpResponse::Unauthorized().json("Unauthorized"),
    }
}

pub async fn update_me(req: HttpRequest, body: web::Json<UpdateProfileRequest>) -> impl Responder {
    match get_current_user(&req) {
        Some(claims) => {
            let mut conn = establish_connection();
            match diesel::update(users::table.find(claims.user_id))
                .set((
                    crate::schema::users::username.eq(body.username.clone().unwrap_or_default()),
                    crate::schema::users::bio.eq(body.bio.clone()),
                    crate::schema::users::avatar.eq(body.avatar.clone()),
                ))
                .execute(&mut conn)
            {
                Ok(_) => HttpResponse::Ok().json("Profile updated"),
                Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
            }
        },
        None => HttpResponse::Unauthorized().json("Unauthorized"),
    }
}

pub async fn update_password(req: HttpRequest, body: web::Json<UpdatePasswordRequest>) -> impl Responder {
    match get_current_user(&req) {
        Some(claims) => {
            let mut conn = establish_connection();
            match users::table.find(claims.user_id).first::<User>(&mut conn) {
                Ok(user) => {
                    if !verify_password(&body.old_password, &user.password_hash).unwrap_or(false) {
                        return HttpResponse::BadRequest().json("Invalid old password");
                    }
                    let new_hash = match hash_password(&body.new_password) {
                        Ok(h) => h,
                        Err(e) => return HttpResponse::InternalServerError().json(format!("Password error: {}", e)),
                    };
                    match diesel::update(users::table.find(claims.user_id))
                        .set(crate::schema::users::password_hash.eq(new_hash))
                        .execute(&mut conn)
                    {
                        Ok(_) => HttpResponse::Ok().json("Password updated"),
                        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
                    }
                },
                Err(_) => HttpResponse::NotFound().json("User not found"),
            }
        },
        None => HttpResponse::Unauthorized().json("Unauthorized"),
    }
}

pub async fn get_posts(query: web::Query<PostFilterQuery>) -> impl Responder {
    let mut conn = establish_connection();
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(10).min(100);
    let offset = (page - 1) * per_page;

    // 处理搜索模式
    let search_pattern = query.q.as_ref().map(|q| format!("%{}%", q));

    let now = Utc::now().naive_utc();
    // 计算总数
    let mut count_query = posts::table
        .filter(posts::deleted_at.is_null())
        .filter(
            posts::status.eq("published")
                .or(posts::is_published.eq(true))
                .or(
                    posts::published_at.is_not_null()
                        .and(posts::published_at.le(now))
                )
        )
        .into_boxed();

    if let Some(category_id) = query.category_id {
        count_query = count_query.filter(posts::category_id.eq(category_id));
    }

    if let Some(ref pattern) = search_pattern {
        count_query = count_query.filter(
            posts::title.like(pattern)
                .or(posts::content.like(pattern))
                .or(posts::excerpt.like(pattern))
        );
    }

    let total: i64 = count_query.count().get_result(&mut conn).unwrap_or(0);
    
    // 查询结果
    let mut results_query = posts::table
        .filter(posts::deleted_at.is_null())
        .filter(
            posts::status.eq("published")
                .or(posts::is_published.eq(true))
                .or(
                    posts::published_at.is_not_null()
                        .and(posts::published_at.le(now))
                )
        )
        .into_boxed();

    if let Some(category_id) = query.category_id {
        results_query = results_query.filter(posts::category_id.eq(category_id));
    }

    if let Some(ref pattern) = search_pattern {
        results_query = results_query.filter(
            posts::title.like(pattern)
                .or(posts::content.like(pattern))
                .or(posts::excerpt.like(pattern))
        );
    }

    let results: Vec<Post> = if let Some("asc") = query.order.as_deref() {
        match query.sort_by.as_deref() {
            Some("title") => results_query.order(posts::title.asc()),
            Some("created_at") => results_query.order(posts::created_at.asc()),
            Some("published_at") => results_query.order(posts::published_at.asc()),
            _ => results_query.order(posts::created_at.asc()),
        }
        .limit(per_page)
        .offset(offset)
        .load(&mut conn)
        .unwrap_or_default()
    } else {
        match query.sort_by.as_deref() {
            Some("title") => results_query.order(posts::title.desc()),
            Some("created_at") => results_query.order(posts::created_at.desc()),
            Some("published_at") => results_query.order(posts::published_at.desc()),
            _ => results_query.order(posts::created_at.desc()),
        }
        .limit(per_page)
        .offset(offset)
        .load(&mut conn)
        .unwrap_or_default()
    };

    let total_pages = (total as f64 / per_page as f64).ceil() as i64;

    HttpResponse::Ok().json(PaginatedResponse {
        data: results,
        page,
        per_page,
        total,
        total_pages,
    })
}

pub async fn get_post(path: web::Path<i32>) -> impl Responder {
    let id = path.into_inner();
    let mut conn = establish_connection();
    match posts::table.find(id).first::<Post>(&mut conn) {
        Ok(post) => {
            if post.deleted_at.is_some() {
                return HttpResponse::NotFound().finish();
            }
            let now = Utc::now().naive_utc();
            if post.status.as_deref() != Some("published") && post.is_published != Some(true) && !(post.published_at.is_some() && post.published_at.unwrap() <= now) {
                return HttpResponse::NotFound().finish();
            }
            let _ = diesel::update(posts::table.find(id))
                .set(posts::view_count.eq(post.view_count.unwrap_or(0) + 1))
                .execute(&mut conn);
            HttpResponse::Ok().json(post)
        },
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub async fn get_post_by_slug(path: web::Path<String>) -> impl Responder {
    let slug = path.into_inner();
    let mut conn = establish_connection();
    match posts::table.filter(posts::slug.eq(&slug)).first::<Post>(&mut conn) {
        Ok(post) => {
            if post.deleted_at.is_some() {
                return HttpResponse::NotFound().finish();
            }
            let now = Utc::now().naive_utc();
            if post.status.as_deref() != Some("published") && post.is_published != Some(true) && !(post.published_at.is_some() && post.published_at.unwrap() <= now) {
                return HttpResponse::NotFound().finish();
            }
            HttpResponse::Ok().json(post)
        },
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub async fn create_post(req: HttpRequest, body: web::Json<CreatePostRequest>) -> impl Responder {
    let user = match get_user_from_request(&req) {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().json("Unauthorized"),
    };

    let mut conn = establish_connection();
    let status = body.status.clone().unwrap_or_else(|| "draft".to_string());
    let now = Utc::now().naive_utc();

    let new_post = NewPost {
        title: body.title.clone(),
        slug: body.slug.clone(),
        content: body.content.clone(),
        excerpt: body.excerpt.clone(),
        author: user.username.clone(),
        status: Some(status.clone()),
        published_at: body.published_at,
        category_id: body.category_id,
        user_id: Some(user.user_id),
        summary: body.summary.clone(),
        cover_image: body.cover_image.clone(),
        is_published: if body.published_at.is_some() { Some(false) } else { body.is_published },
        is_top: body.is_top,
        allow_comments: body.allow_comments,
        seo_title: body.seo_title.clone(),
        seo_description: body.seo_description.clone(),
        seo_keywords: body.seo_keywords.clone(),
        seo_canonical: body.seo_canonical.clone(),
        seo_robots: body.seo_robots.clone(),
    };

    let result = diesel::insert_into(posts::table)
        .values(&new_post)
        .execute(&mut conn);

    match result {
        Ok(_) => {
            let created_post: Post = posts::table.order(posts::id.desc()).first(&mut conn).unwrap();
            if let Some(tag_ids) = &body.tag_ids {
                for tag_id in tag_ids {
                    let post_tag = PostTag {
                        post_id: created_post.id,
                        tag_id: *tag_id,
                    };
                    let _ = diesel::insert_into(post_tags::table)
                        .values(&post_tag)
                        .execute(&mut conn);
                }
            }
            HttpResponse::Created().json(created_post)
        },
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
    }
}

pub async fn update_post(req: HttpRequest, path: web::Path<i32>, body: web::Json<UpdatePostRequest>) -> impl Responder {
    let user = match get_user_from_request(&req) {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().json("Unauthorized"),
    };

    let id = path.into_inner();
    let mut conn = establish_connection();

    match posts::table.find(id).first::<Post>(&mut conn) {
        Ok(post) => {
            if post.user_id != Some(user.user_id) && user.role != "admin" {
                return HttpResponse::Forbidden().json("Cannot edit another user's post");
            }

            // 保存当前版本到 post_versions 表
            let max_version: Option<i32> = post_versions::table
                .filter(post_versions::post_id.eq(id))
                .select(post_versions::version_number)
                .order(post_versions::version_number.desc())
                .first(&mut conn)
                .optional()
                .unwrap_or(None);

            let version_number = max_version.map(|v| v + 1).unwrap_or(1);

            let new_version = NewPostVersion {
                post_id: post.id,
                title: post.title.clone(),
                slug: post.slug.clone(),
                content: post.content.clone(),
                excerpt: post.excerpt.clone(),
                author: post.author.clone(),
                status: post.status.clone(),
                created_at: post.created_at,
                updated_at: post.updated_at,
                published_at: post.published_at,
                category_id: post.category_id,
                user_id: post.user_id,
                summary: post.summary.clone(),
                cover_image: post.cover_image.clone(),
                is_published: post.is_published,
                is_top: post.is_top,
                allow_comments: post.allow_comments,
                version_number,
                seo_title: post.seo_title.clone(),
                seo_description: post.seo_description.clone(),
                seo_keywords: post.seo_keywords.clone(),
                seo_canonical: post.seo_canonical.clone(),
                seo_robots: post.seo_robots.clone(),
            };

            let _ = diesel::insert_into(post_versions::table)
                .values(&new_version)
                .execute(&mut conn);

            let update_data = UpdatePost {
                title: body.title.clone(),
                slug: body.slug.clone(),
                content: body.content.clone(),
                excerpt: body.excerpt.clone(),
                author: None,
                status: body.status.clone(),
                published_at: body.published_at,
                deleted_at: None,
                category_id: body.category_id,
                user_id: None,
                summary: body.summary.clone(),
                cover_image: body.cover_image.clone(),
                is_published: if body.published_at.is_some() { Some(false) } else { body.is_published },
                is_top: body.is_top,
                allow_comments: body.allow_comments,
                view_count: None,
                seo_title: body.seo_title.clone(),
                seo_description: body.seo_description.clone(),
                seo_keywords: body.seo_keywords.clone(),
                seo_canonical: body.seo_canonical.clone(),
                seo_robots: body.seo_robots.clone(),
            };

            match diesel::update(posts::table.find(id))
                .set(&update_data)
                .execute(&mut conn)
            {
                Ok(_) => {
                    if let Some(tag_ids) = &body.tag_ids {
                        let _ = diesel::delete(post_tags::table.filter(post_tags::post_id.eq(id)))
                            .execute(&mut conn);
                        for tag_id in tag_ids {
                            let post_tag = PostTag {
                                post_id: id,
                                tag_id: *tag_id,
                            };
                            let _ = diesel::insert_into(post_tags::table)
                                .values(&post_tag)
                                .execute(&mut conn);
                        }
                    }
                    HttpResponse::Ok().json("Post updated")
                },
                Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
            }
        },
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub async fn delete_post(req: HttpRequest, path: web::Path<i32>) -> impl Responder {
    let user = match get_user_from_request(&req) {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().json("Unauthorized"),
    };

    let id = path.into_inner();
    let mut conn = establish_connection();

    match posts::table.find(id).first::<Post>(&mut conn) {
        Ok(post) => {
            if post.user_id != Some(user.user_id) && user.role != "admin" {
                return HttpResponse::Forbidden().json("Cannot delete another user's post");
            }

            match diesel::update(posts::table.find(id))
                .set(posts::deleted_at.eq(Some(Utc::now().naive_utc())))
                .execute(&mut conn)
            {
                Ok(_) => HttpResponse::Ok().json("Post deleted"),
                Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
            }
        },
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub async fn update_post_status(req: HttpRequest, path: web::Path<i32>, body: web::Json<UpdateStatusRequest>) -> impl Responder {
    let user = match get_user_from_request(&req) {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().json("Unauthorized"),
    };

    let id = path.into_inner();
    let mut conn = establish_connection();

    match posts::table.find(id).first::<Post>(&mut conn) {
        Ok(post) => {
            if post.user_id != Some(user.user_id) && user.role != "admin" {
                return HttpResponse::Forbidden().json("Cannot update another user's post");
            }

            let now = Utc::now().naive_utc();
            let published_at = if body.status == "published" { Some(now) } else { None };

            match diesel::update(posts::table.find(id))
                .set((
                    posts::status.eq(&body.status),
                    posts::published_at.eq(published_at),
                    posts::is_published.eq(body.status == "published"),
                ))
                .execute(&mut conn)
            {
                Ok(_) => HttpResponse::Ok().json("Post status updated"),
                Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
            }
        },
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub async fn get_categories() -> impl Responder {
    let mut conn = establish_connection();
    let results = categories::table.load::<Category>(&mut conn).unwrap_or_default();
    HttpResponse::Ok().json(results)
}

pub async fn get_category(path: web::Path<i32>) -> impl Responder {
    let id = path.into_inner();
    let mut conn = establish_connection();
    match categories::table.find(id).first::<Category>(&mut conn) {
        Ok(category) => HttpResponse::Ok().json(category),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub async fn create_category(req: HttpRequest, body: web::Json<CategoryRequest>) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let mut conn = establish_connection();

    if categories::table.filter(categories::name.eq(&body.name)).first::<Category>(&mut conn).is_ok() {
        return HttpResponse::BadRequest().json("Category already exists");
    }

    let new_category = NewCategory {
        name: body.name.clone(),
        slug: body.slug.clone(),
        description: body.description.clone(),
        parent_id: body.parent_id,
    };

    match diesel::insert_into(categories::table)
        .values(&new_category)
        .execute(&mut conn)
    {
        Ok(_) => {
            let created: Category = categories::table.order(categories::id.desc()).first(&mut conn).unwrap();
            HttpResponse::Created().json(created)
        },
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
    }
}

pub async fn update_category(req: HttpRequest, path: web::Path<i32>, body: web::Json<UpdateCategory>) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let id = path.into_inner();
    let mut conn = establish_connection();

    match diesel::update(categories::table.find(id))
        .set(&*body)
        .execute(&mut conn)
    {
        Ok(affected) if affected > 0 => HttpResponse::Ok().json("Category updated"),
        _ => HttpResponse::NotFound().finish(),
    }
}

pub async fn delete_category(req: HttpRequest, path: web::Path<i32>) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let id = path.into_inner();
    let mut conn = establish_connection();

    match diesel::delete(categories::table.find(id))
        .execute(&mut conn)
    {
        Ok(affected) if affected > 0 => HttpResponse::Ok().json("Category deleted"),
        _ => HttpResponse::NotFound().finish(),
    }
}

pub async fn get_category_posts(path: web::Path<i32>, query: web::Query<PaginationQuery>) -> impl Responder {
    let category_id = path.into_inner();
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(10).min(100);
    let offset = (page - 1) * per_page;

    let mut conn = establish_connection();

    let now = Utc::now().naive_utc();
    
    // 为计数创建查询
    let count_query = posts::table
        .filter(posts::category_id.eq(category_id))
        .filter(posts::deleted_at.is_null())
        .filter(
            posts::status.eq("published")
                .or(posts::is_published.eq(true))
                .or(
                    posts::published_at.is_not_null()
                        .and(posts::published_at.le(now))
                )
        );

    let total: i64 = count_query.count().get_result(&mut conn).unwrap_or(0);
    
    // 为结果创建查询
    let results: Vec<Post> = posts::table
        .filter(posts::category_id.eq(category_id))
        .filter(posts::deleted_at.is_null())
        .filter(
            posts::status.eq("published")
                .or(posts::is_published.eq(true))
                .or(
                    posts::published_at.is_not_null()
                        .and(posts::published_at.le(now))
                )
        )
        .order(posts::created_at.desc())
        .limit(per_page)
        .offset(offset)
        .load(&mut conn)
        .unwrap_or_default();

    let total_pages = (total as f64 / per_page as f64).ceil() as i64;

    HttpResponse::Ok().json(PaginatedResponse {
        data: results,
        page,
        per_page,
        total,
        total_pages,
    })
}

pub async fn get_tags() -> impl Responder {
    let mut conn = establish_connection();
    let results = tags::table.load::<Tag>(&mut conn).unwrap_or_default();
    HttpResponse::Ok().json(results)
}

pub async fn create_tag(req: HttpRequest, body: web::Json<TagRequest>) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let mut conn = establish_connection();

    if tags::table.filter(tags::name.eq(&body.name)).first::<Tag>(&mut conn).is_ok() {
        return HttpResponse::BadRequest().json("Tag already exists");
    }

    let new_tag = NewTag {
        name: body.name.clone(),
        slug: body.slug.clone(),
    };

    match diesel::insert_into(tags::table)
        .values(&new_tag)
        .execute(&mut conn)
    {
        Ok(_) => {
            let created: Tag = tags::table.order(tags::id.desc()).first(&mut conn).unwrap();
            HttpResponse::Created().json(created)
        },
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
    }
}

pub async fn update_tag(req: HttpRequest, path: web::Path<i32>, body: web::Json<UpdateTag>) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let id = path.into_inner();
    let mut conn = establish_connection();

    match diesel::update(tags::table.find(id))
        .set(&*body)
        .execute(&mut conn)
    {
        Ok(affected) if affected > 0 => HttpResponse::Ok().json("Tag updated"),
        _ => HttpResponse::NotFound().finish(),
    }
}

pub async fn delete_tag(req: HttpRequest, path: web::Path<i32>) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let id = path.into_inner();
    let mut conn = establish_connection();

    match diesel::delete(tags::table.find(id))
        .execute(&mut conn)
    {
        Ok(affected) if affected > 0 => HttpResponse::Ok().json("Tag deleted"),
        _ => HttpResponse::NotFound().finish(),
    }
}

pub async fn get_tag_posts(path: web::Path<i32>, query: web::Query<PaginationQuery>) -> impl Responder {
    let tag_id = path.into_inner();
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(10).min(100);
    let offset = (page - 1) * per_page;

    let mut conn = establish_connection();

    let now = Utc::now().naive_utc();
    
    // 为计数创建查询
    let count_query = posts::table
        .inner_join(post_tags::table.on(posts::id.eq(post_tags::post_id)))
        .filter(post_tags::tag_id.eq(tag_id))
        .filter(posts::deleted_at.is_null())
        .filter(
            posts::status.eq("published")
                .or(posts::is_published.eq(true))
                .or(
                    posts::published_at.is_not_null()
                        .and(posts::published_at.le(now))
                )
        );

    let total: i64 = count_query.count().get_result(&mut conn).unwrap_or(0);
    
    // 为结果创建查询
    let results: Vec<Post> = posts::table
        .inner_join(post_tags::table.on(posts::id.eq(post_tags::post_id)))
        .filter(post_tags::tag_id.eq(tag_id))
        .filter(posts::deleted_at.is_null())
        .filter(
            posts::status.eq("published")
                .or(posts::is_published.eq(true))
                .or(
                    posts::published_at.is_not_null()
                        .and(posts::published_at.le(now))
                )
        )
        .select(posts::all_columns)
        .order(posts::created_at.desc())
        .limit(per_page)
        .offset(offset)
        .load(&mut conn)
        .unwrap_or_default();

    let total_pages = (total as f64 / per_page as f64).ceil() as i64;

    HttpResponse::Ok().json(PaginatedResponse {
        data: results,
        page,
        per_page,
        total,
        total_pages,
    })
}

pub async fn get_comments(path: web::Path<i32>) -> impl Responder {
    let post_id = path.into_inner();
    let mut conn = establish_connection();

    let results = comments::table
        .filter(comments::post_id.eq(post_id))
        .filter(comments::status.eq("approved"))
        .order(comments::created_at.desc())
        .load::<Comment>(&mut conn)
        .unwrap_or_default();

    HttpResponse::Ok().json(results)
}

pub async fn create_comment(req: HttpRequest, body: web::Json<CommentRequest>) -> impl Responder {
    let user = get_user_from_request(&req);

    let mut conn = establish_connection();

    if posts::table.find(body.post_id).first::<Post>(&mut conn).is_err() {
        return HttpResponse::NotFound().json("Post not found");
    }

    let (user_id, status_str) = match user {
        Some(u) => (Some(u.user_id), "approved".to_string()),
        None => (None, "pending".to_string()),
    };

    let new_comment = NewComment {
        post_id: body.post_id,
        user_id,
        parent_id: body.parent_id,
        content: body.content.clone(),
        author_name: body.author_name.clone(),
        author_email: body.author_email.clone(),
        author_website: body.author_website.clone(),
        status: status_str.clone(),
    };

    match diesel::insert_into(comments::table)
        .values(&new_comment)
        .execute(&mut conn)
    {
        Ok(_) => {
            let message = if status_str == "pending" {
                "Comment created successfully, pending approval"
            } else {
                "Comment created successfully"
            };
            HttpResponse::Created().json(serde_json::json!({"message": message}))
        },
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
    }
}

pub async fn approve_comment(req: HttpRequest, path: web::Path<i32>) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let comment_id = path.into_inner();
    let mut conn = establish_connection();

    match diesel::update(comments::table.find(comment_id))
        .set(comments::status.eq("approved"))
        .execute(&mut conn)
    {
        Ok(affected) if affected > 0 => HttpResponse::Ok().json("Comment approved"),
        _ => HttpResponse::NotFound().finish(),
    }
}

pub async fn delete_comment(req: HttpRequest, path: web::Path<i32>) -> impl Responder {
    let user = match get_user_from_request(&req) {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().json("Unauthorized"),
    };

    let comment_id = path.into_inner();
    let mut conn = establish_connection();

    match comments::table.find(comment_id).first::<Comment>(&mut conn) {
        Ok(comment) => {
            if comment.user_id != Some(user.user_id) && user.role != "admin" {
                return HttpResponse::Forbidden().json("Cannot delete another user's comment");
            }

            match diesel::delete(comments::table.find(comment_id))
                .execute(&mut conn)
            {
                Ok(_) => HttpResponse::Ok().json("Comment deleted"),
                Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
            }
        },
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub async fn upload_media(req: HttpRequest, mut payload: Multipart) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let user = match get_user_from_request(&req) {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().json("Unauthorized"),
    };

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let filename = content_disposition
            .and_then(|cd| cd.get_filename())
            .unwrap_or("file")
            .to_string();

        let mut data = Vec::new();
        while let Ok(Some(chunk)) = field.try_next().await {
            data.extend_from_slice(&chunk);
        }

        let filepath = format!("./uploads/{}", filename);
        let mimetype = field
            .content_type()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        let mut conn = establish_connection();
        let new_media = NewMedia {
            filename: filename.clone(),
            filepath: filepath.clone(),
            mimetype,
            size: data.len() as i64,
            uploaded_by: Some(user.user_id),
            category: None,
        };

        match diesel::insert_into(media::table)
            .values(&new_media)
            .execute(&mut conn)
        {
            Ok(_) => {
                if let Ok(_) = std::fs::create_dir_all("./uploads") {
                    let _ = std::fs::write(&filepath, &data);
                }
                return HttpResponse::Created().json(serde_json::json!({
                    "filename": filename,
                    "filepath": filepath
                }));
            },
            Err(e) => return HttpResponse::InternalServerError().json(format!("Error: {}", e)),
        }
    }

    HttpResponse::BadRequest().json("No file uploaded")
}

pub async fn get_media(req: HttpRequest) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let mut conn = establish_connection();
    let results = media::table.load::<Media>(&mut conn).unwrap_or_default();
    HttpResponse::Ok().json(results)
}

pub async fn delete_media(req: HttpRequest, path: web::Path<i32>) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let id = path.into_inner();
    let mut conn = establish_connection();

    match diesel::delete(media::table.find(id))
        .execute(&mut conn)
    {
        Ok(_) => HttpResponse::Ok().json("Media deleted"),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub async fn get_media_categories(req: HttpRequest) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let mut conn = establish_connection();
    let categories: Vec<String> = media::table
        .filter(media::category.is_not_null())
        .select(media::category)
        .distinct()
        .load(&mut conn)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|c| c)
        .collect();

    HttpResponse::Ok().json(categories)
}

#[derive(serde::Deserialize)]
pub struct MediaSearchQuery {
    pub q: Option<String>,
    pub category: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn search_media(req: HttpRequest, query: web::Query<MediaSearchQuery>) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let mut conn = establish_connection();
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(10).min(100);
    let offset = (page - 1) * per_page;

    // 构建搜索模式
    let search_pattern = query.q.as_ref().map(|q| format!("%{}%", q));

    // 构建计数查询
    let mut count_query = media::table.into_boxed();
    if let Some(ref pattern) = search_pattern {
        count_query = count_query.filter(
            media::filename.like(pattern)
                .or(media::mimetype.like(pattern))
        );
    }
    if let Some(category) = &query.category {
        count_query = count_query.filter(media::category.eq(category));
    }
    let total: i64 = count_query.count().get_result(&mut conn).unwrap_or(0);

    // 构建结果查询
    let mut results_query = media::table.into_boxed();
    if let Some(ref pattern) = search_pattern {
        results_query = results_query.filter(
            media::filename.like(pattern)
                .or(media::mimetype.like(pattern))
        );
    }
    if let Some(category) = &query.category {
        results_query = results_query.filter(media::category.eq(category));
    }
    let results: Vec<Media> = results_query
        .order(media::created_at.desc())
        .limit(per_page)
        .offset(offset)
        .load(&mut conn)
        .unwrap_or_default();

    let total_pages = (total as f64 / per_page as f64).ceil() as i64;

    HttpResponse::Ok().json(PaginatedResponse {
        data: results,
        page,
        per_page,
        total,
        total_pages,
    })
}

#[derive(serde::Deserialize)]
pub struct BatchMediaRequest {
    pub media_ids: Vec<i32>,
    pub category: Option<String>,
}

pub async fn batch_update_media(req: HttpRequest, body: web::Json<BatchMediaRequest>) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let mut conn = establish_connection();

    match diesel::update(media::table.filter(media::id.eq_any(&body.media_ids)))
        .set(media::category.eq(body.category.clone()))
        .execute(&mut conn)
    {
        Ok(affected) if affected > 0 => HttpResponse::Ok().json(serde_json::json!({
            "message": "Media updated successfully",
            "affected": affected
        })),
        _ => HttpResponse::NotFound().json("No media found to update"),
    }
}

pub async fn batch_delete_media(req: HttpRequest, body: web::Json<BatchMediaRequest>) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let mut conn = establish_connection();

    match diesel::delete(media::table.filter(media::id.eq_any(&body.media_ids)))
        .execute(&mut conn)
    {
        Ok(affected) if affected > 0 => HttpResponse::Ok().json(serde_json::json!({
            "message": "Media deleted successfully",
            "affected": affected
        })),
        _ => HttpResponse::NotFound().json("No media found to delete"),
    }
}

pub async fn search(query: web::Query<SearchQuery>) -> impl Responder {
    let mut conn = establish_connection();
    let search_pattern = format!("%{}%", query.q);

    let now = Utc::now().naive_utc();
    let results: Vec<Post> = posts::table
        .filter(posts::deleted_at.is_null())
        .filter(
            posts::status.eq("published")
                .or(posts::is_published.eq(true))
                .or(
                    posts::published_at.is_not_null()
                        .and(posts::published_at.le(now))
                )
        )
        .filter(
            posts::title.like(&search_pattern)
                .or(posts::content.like(&search_pattern))
                .or(posts::excerpt.like(&search_pattern))
        )
        .order(posts::created_at.desc())
        .limit(50)
        .load(&mut conn)
        .unwrap_or_default();

    HttpResponse::Ok().json(results)
}

pub async fn add_post_tags(req: HttpRequest, body: web::Json<PostTagsRequest>) -> impl Responder {
    let user = match get_user_from_request(&req) {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().json("Unauthorized"),
    };

    let mut conn = establish_connection();

    match posts::table.find(body.post_id).first::<Post>(&mut conn) {
        Ok(post) => {
            if post.user_id != Some(user.user_id) && user.role != "admin" {
                return HttpResponse::Forbidden().json("Cannot modify another user's post");
            }
        },
        Err(_) => return HttpResponse::NotFound().json("Post not found"),
    }

    diesel::delete(post_tags::table.filter(post_tags::post_id.eq(body.post_id)))
        .execute(&mut conn)
        .expect("Error deleting existing post tags");

    for tag_id in &body.tag_ids {
        if tags::table.find(*tag_id).first::<Tag>(&mut conn).is_err() {
            continue;
        }

        let post_tag = PostTag {
            post_id: body.post_id,
            tag_id: *tag_id,
        };

        let _ = diesel::insert_into(post_tags::table)
            .values(&post_tag)
            .execute(&mut conn);
    }

    HttpResponse::Ok().json("Post tags updated")
}

pub async fn get_admin_users(req: HttpRequest, query: web::Query<UserFilterQuery>) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let mut conn = establish_connection();
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(10).min(100);
    let offset = (page - 1) * per_page;

    let results: Vec<User>;
    let total: i64;

    if let Some(q) = &query.q {
        let search_pattern = format!("%{}%", q);
        total = users::table
            .filter(
                users::username.like(&search_pattern)
                    .or(users::email.like(&search_pattern))
            )
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);
        results = users::table
            .filter(
                users::username.like(&search_pattern)
                    .or(users::email.like(&search_pattern))
            )
            .order(users::id.desc())
            .limit(per_page)
            .offset(offset)
            .load(&mut conn)
            .unwrap_or_default();
    } else {
        total = users::table.count().get_result(&mut conn).unwrap_or(0);
        results = users::table
            .order(users::id.desc())
            .limit(per_page)
            .offset(offset)
            .load(&mut conn)
            .unwrap_or_default();
    }

    let total_pages = (total as f64 / per_page as f64).ceil() as i64;

    HttpResponse::Ok().json(PaginatedResponse {
        data: results,
        page,
        per_page,
        total,
        total_pages,
    })
}

pub async fn update_admin_user(req: HttpRequest, path: web::Path<i32>, body: web::Json<UpdateUserRequest>) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let id = path.into_inner();
    let mut conn = establish_connection();

    match users::table.find(id).first::<User>(&mut conn) {
        Ok(_) => {
            let update_data = (
                users::username.eq(body.username.clone().unwrap_or_default()),
                users::email.eq(body.email.clone().unwrap_or_default()),
                users::role.eq(body.role.clone().unwrap_or("user".to_string())),
                users::bio.eq(body.bio.clone()),
                users::avatar.eq(body.avatar.clone()),
            );

            match diesel::update(users::table.find(id))
                .set(update_data)
                .execute(&mut conn)
            {
                Ok(affected) if affected > 0 => HttpResponse::Ok().json("User updated"),
                _ => HttpResponse::NotFound().finish(),
            }
        },
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub async fn delete_admin_user(req: HttpRequest, path: web::Path<i32>) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let id = path.into_inner();
    let mut conn = establish_connection();

    match diesel::delete(users::table.find(id))
        .execute(&mut conn)
    {
        Ok(affected) if affected > 0 => HttpResponse::Ok().json("User deleted"),
        _ => HttpResponse::NotFound().finish(),
    }
}

pub async fn get_admin_comments(req: HttpRequest, query: web::Query<CommentFilterQuery>) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let mut conn = establish_connection();
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(10).min(100);
    let offset = (page - 1) * per_page;

    let mut db_query = comments::table.into_boxed();

    if let Some(status) = &query.status {
        db_query = db_query.filter(comments::status.eq(status));
    }

    let total: i64 = db_query.count().get_result(&mut conn).unwrap_or(0);
    
    let mut results_query = comments::table.into_boxed();
    
    if let Some(status) = &query.status {
        results_query = results_query.filter(comments::status.eq(status));
    }
    
    let results: Vec<Comment> = results_query
        .order(comments::created_at.desc())
        .limit(per_page)
        .offset(offset)
        .load(&mut conn)
        .unwrap_or_default();

    let total_pages = (total as f64 / per_page as f64).ceil() as i64;

    HttpResponse::Ok().json(PaginatedResponse {
        data: results,
        page,
        per_page,
        total,
        total_pages,
    })
}

pub async fn batch_approve_comments(req: HttpRequest, body: web::Json<BatchCommentRequest>) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let mut conn = establish_connection();

    match diesel::update(comments::table.filter(comments::id.eq_any(&body.comment_ids)))
        .set(comments::status.eq("approved"))
        .execute(&mut conn)
    {
        Ok(affected) if affected > 0 => HttpResponse::Ok().json(serde_json::json!({
            "message": "Comments approved successfully",
            "affected": affected
        })),
        _ => HttpResponse::NotFound().json("No comments found to approve"),
    }
}

pub async fn batch_delete_comments(req: HttpRequest, body: web::Json<BatchCommentRequest>) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let mut conn = establish_connection();

    match diesel::delete(comments::table.filter(comments::id.eq_any(&body.comment_ids)))
        .execute(&mut conn)
    {
        Ok(affected) if affected > 0 => HttpResponse::Ok().json(serde_json::json!({
            "message": "Comments deleted successfully",
            "affected": affected
        })),
        _ => HttpResponse::NotFound().json("No comments found to delete"),
    }
}

pub async fn get_stats_overview(req: HttpRequest) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let mut conn = establish_connection();

    // 统计文章数量
    let post_count: i64 = posts::table
        .filter(posts::deleted_at.is_null())
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    // 统计用户数量
    let user_count: i64 = users::table.count().get_result(&mut conn).unwrap_or(0);

    // 统计评论数量
    let comment_count: i64 = comments::table.count().get_result(&mut conn).unwrap_or(0);

    // 统计分类数量
    let category_count: i64 = categories::table.count().get_result(&mut conn).unwrap_or(0);

    // 统计标签数量
    let tag_count: i64 = tags::table.count().get_result(&mut conn).unwrap_or(0);

    HttpResponse::Ok().json(serde_json::json!({
        "posts": post_count,
        "users": user_count,
        "comments": comment_count,
        "categories": category_count,
        "tags": tag_count
    }))
}

pub async fn get_stats_visits(req: HttpRequest) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let mut conn = establish_connection();

    let now = Utc::now().naive_utc();
    // 获取热门文章（按访问量排序）
    let popular_posts: Vec<Post> = posts::table
        .filter(posts::deleted_at.is_null())
        .filter(
            posts::status.eq("published")
                .or(posts::is_published.eq(true))
                .or(
                    posts::published_at.is_not_null()
                        .and(posts::published_at.le(now))
                )
        )
        .order(posts::view_count.desc())
        .limit(10)
        .load(&mut conn)
        .unwrap_or_default();

    // 简化热门文章数据
    let simplified_posts: Vec<serde_json::Value> = popular_posts.into_iter().map(|post| {
        serde_json::json!({
            "id": post.id,
            "title": post.title,
            "view_count": post.view_count.unwrap_or(0)
        })
    }).collect();

    HttpResponse::Ok().json(serde_json::json!({
        "popular_posts": simplified_posts
    }))
}

pub async fn get_stats_users(req: HttpRequest) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let mut conn = establish_connection();

    // 获取最近注册用户
    let recent_users: Vec<User> = users::table
        .order(users::created_at.desc())
        .limit(10)
        .load(&mut conn)
        .unwrap_or_default();

    // 简化最近用户数据
    let simplified_users: Vec<serde_json::Value> = recent_users.into_iter().map(|user| {
        serde_json::json!({
            "id": user.id,
            "username": user.username,
            "email": user.email,
            "role": user.role,
            "created_at": user.created_at
        })
    }).collect();

    HttpResponse::Ok().json(serde_json::json!({
        "recent_users": simplified_users
    }))
}

pub async fn get_post_versions(req: HttpRequest, path: web::Path<i32>) -> impl Responder {
    let user = match get_user_from_request(&req) {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().json("Unauthorized"),
    };

    let post_id = path.into_inner();
    let mut conn = establish_connection();

    // 检查文章是否存在且用户有权限
    match posts::table.find(post_id).first::<Post>(&mut conn) {
        Ok(post) => {
            if post.user_id != Some(user.user_id) && user.role != "admin" {
                return HttpResponse::Forbidden().json("Cannot access another user's post");
            }

            // 获取文章的历史版本
            let versions: Vec<PostVersion> = post_versions::table
                .filter(post_versions::post_id.eq(post_id))
                .order(post_versions::version_number.desc())
                .load(&mut conn)
                .unwrap_or_default();

            HttpResponse::Ok().json(versions)
        },
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub async fn rollback_post_version(req: HttpRequest, path: web::Path<(i32, i32)>) -> impl Responder {
    let user = match get_user_from_request(&req) {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().json("Unauthorized"),
    };

    let (post_id, version_id) = path.into_inner();
    let mut conn = establish_connection();

    // 检查文章是否存在且用户有权限
    match posts::table.find(post_id).first::<Post>(&mut conn) {
        Ok(post) => {
            if post.user_id != Some(user.user_id) && user.role != "admin" {
                return HttpResponse::Forbidden().json("Cannot modify another user's post");
            }

            // 查找指定版本
            match post_versions::table
                .filter(post_versions::post_id.eq(post_id))
                .filter(post_versions::id.eq(version_id))
                .first::<PostVersion>(&mut conn)
            {
                Ok(version) => {
                    // 保存当前版本到 post_versions 表
                    let max_version: Option<i32> = post_versions::table
                        .filter(post_versions::post_id.eq(post_id))
                        .select(post_versions::version_number)
                        .order(post_versions::version_number.desc())
                        .first(&mut conn)
                        .optional()
                        .unwrap_or(None);

                    let new_version_number = max_version.map(|v| v + 1).unwrap_or(1);

                    let current_version = NewPostVersion {
                        post_id: post.id,
                        title: post.title.clone(),
                        slug: post.slug.clone(),
                        content: post.content.clone(),
                        excerpt: post.excerpt.clone(),
                        author: post.author.clone(),
                        status: post.status.clone(),
                        created_at: post.created_at,
                        updated_at: post.updated_at,
                        published_at: post.published_at,
                        category_id: post.category_id,
                        user_id: post.user_id,
                        summary: post.summary.clone(),
                        cover_image: post.cover_image.clone(),
                        is_published: post.is_published,
                        is_top: post.is_top,
                        allow_comments: post.allow_comments,
                        version_number: new_version_number,
                        seo_title: post.seo_title.clone(),
                        seo_description: post.seo_description.clone(),
                        seo_keywords: post.seo_keywords.clone(),
                        seo_canonical: post.seo_canonical.clone(),
                        seo_robots: post.seo_robots.clone(),
                    };

                    let _ = diesel::insert_into(post_versions::table)
                        .values(&current_version)
                        .execute(&mut conn);

                    // 回滚到指定版本
                    let update_data = UpdatePost {
                        title: Some(version.title.clone()),
                        slug: version.slug.clone(),
                        content: Some(version.content.clone()),
                        excerpt: version.excerpt.clone(),
                        author: Some(version.author.clone()),
                        status: version.status.clone(),
                        published_at: version.published_at,
                        deleted_at: None,
                        category_id: version.category_id,
                        user_id: version.user_id,
                        summary: version.summary.clone(),
                        cover_image: version.cover_image.clone(),
                        is_published: version.is_published,
                        is_top: version.is_top,
                        allow_comments: version.allow_comments,
                        view_count: None,
                        seo_title: version.seo_title.clone(),
                        seo_description: version.seo_description.clone(),
                        seo_keywords: version.seo_keywords.clone(),
                        seo_canonical: version.seo_canonical.clone(),
                        seo_robots: version.seo_robots.clone(),
                    };

                    match diesel::update(posts::table.find(post_id))
                        .set(&update_data)
                        .execute(&mut conn)
                    {
                        Ok(_) => HttpResponse::Ok().json("Post rolled back successfully"),
                        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
                    }
                },
                Err(_) => HttpResponse::NotFound().json("Version not found"),
            }
        },
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

// SEO 相关处理函数

// 生成站点地图
pub async fn generate_sitemap() -> impl Responder {
    let mut conn = establish_connection();
    
    // 获取所有已发布的文章
    let published_posts = posts::table
        .filter(posts::is_published.eq(true))
        .order(posts::updated_at.desc())
        .load::<Post>(&mut conn)
        .unwrap_or_default();
    
    // 生成站点地图 XML
    let mut sitemap = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
"#.to_string();
    
    // 添加首页
    sitemap.push_str(r#"  <url>
    <loc>https://example.com</loc>
    <lastmod>"#);
    sitemap.push_str(&Utc::now().format("%Y-%m-%d").to_string());
    sitemap.push_str(r#"</lastmod>
    <changefreq>daily</changefreq>
    <priority>1.0</priority>
  </url>
"#);
    
    // 添加文章页面
    for post in published_posts {
        sitemap.push_str(r#"  <url>
    <loc>https://example.com/posts/"#);
        if let Some(slug) = post.slug {
            sitemap.push_str(&slug);
        } else {
            sitemap.push_str(&post.id.to_string());
        }
        sitemap.push_str(r#"</loc>
    <lastmod>"#);
        sitemap.push_str(&post.updated_at.format("%Y-%m-%d").to_string());
        sitemap.push_str(r#"</lastmod>
    <changefreq>weekly</changefreq>
    <priority>0.8</priority>
  </url>
"#);
    }
    
    sitemap.push_str(r#"</urlset>"#);
    
    HttpResponse::Ok()
        .content_type("application/xml")
        .body(sitemap)
}

// 生成 robots.txt
pub async fn generate_robots() -> impl Responder {
    let robots_txt = r#"User-agent: *
Allow: /

Sitemap: https://example.com/sitemap.xml
"#;
    
    HttpResponse::Ok()
        .content_type("text/plain")
        .body(robots_txt)
}

// 更新文章的 SEO 元数据
pub async fn update_post_seo_metadata(
    req: HttpRequest,
    post_id: web::Path<i32>,
    web::Json(seo_data): web::Json<UpdatePost>,
) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Unauthorized().json("Unauthorized");
    }
    
    let mut conn = establish_connection();
    
    match diesel::update(posts::table.find(post_id.into_inner()))
        .set(&seo_data)
        .execute(&mut conn)
    {
        Ok(_) => HttpResponse::Ok().json("SEO metadata updated successfully"),
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
    }
}
