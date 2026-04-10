use actix_web::{web, HttpResponse, Responder, HttpRequest, HttpMessage};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::{models::{Post, NewPost, UpdatePost, User, NewUser, Comment, NewComment, UpdateComment, Category, NewCategory, UpdateCategory, Tag, NewTag, UpdateTag, PostTag, Media, NewMedia}, schema::posts, schema::users, schema::comments, schema::categories, schema::tags, schema::post_tags, schema::media, db::establish_connection, auth::{generate_token, hash_password, verify_password, verify_token, Claims}};
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
    pub tag_ids: Option<Vec<i32>>,
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
    pub tag_ids: Option<Vec<i32>>,
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

    let mut db_query = posts::table
        .filter(posts::deleted_at.is_null())
        .filter(posts::status.eq("published").or(posts::is_published.eq(true)))
        .into_boxed();

    if let Some(category_id) = query.category_id {
        db_query = db_query.filter(posts::category_id.eq(category_id));
    }

    if let Some(q) = &query.q {
        let search_pattern = format!("%{}%", q);
        db_query = db_query.filter(
            posts::title.like(&search_pattern)
                .or(posts::content.like(&search_pattern))
                .or(posts::excerpt.like(&search_pattern))
        );
    }

    let sort_column = match query.sort_by.as_deref() {
        Some("title") => posts::title,
        Some("created_at") => posts::created_at,
        Some("published_at") => posts::published_at,
        _ => posts::created_at,
    };

    let sort_order = match query.order.as_deref() {
        Some("asc") => diesel::dsl::Asc,
        _ => diesel::dsl::Desc,
    };

    let total: i64 = db_query.clone().count().get_result(&mut conn).unwrap_or(0);
    let results: Vec<Post> = db_query
        .order(sort_column.desc())
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

pub async fn get_post(path: web::Path<i32>) -> impl Responder {
    let id = path.into_inner();
    let mut conn = establish_connection();
    match posts::table.find(id).first::<Post>(&mut conn) {
        Ok(post) => {
            if post.deleted_at.is_some() {
                return HttpResponse::NotFound().finish();
            }
            if post.status.as_deref() != Some("published") && post.is_published != Some(true) {
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
            if post.status.as_deref() != Some("published") && post.is_published != Some(true) {
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
        category_id: body.category_id,
        user_id: Some(user.user_id),
        summary: body.summary.clone(),
        cover_image: body.cover_image.clone(),
        is_published: body.is_published,
        is_top: body.is_top.unwrap_or(false),
        allow_comments: body.allow_comments.unwrap_or(true),
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

            let update_data = UpdatePost {
                title: body.title.clone(),
                slug: body.slug.clone(),
                content: body.content.clone(),
                excerpt: body.excerpt.clone(),
                author: None,
                status: body.status.clone(),
                published_at: None,
                deleted_at: None,
                category_id: body.category_id,
                user_id: None,
                summary: body.summary.clone(),
                cover_image: body.cover_image.clone(),
                is_published: body.is_published,
                is_top: body.is_top,
                allow_comments: body.allow_comments,
                view_count: None,
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

    let mut db_query = posts::table
        .filter(posts::category_id.eq(category_id))
        .filter(posts::deleted_at.is_null())
        .filter(posts::status.eq("published").or(posts::is_published.eq(true)))
        .into_boxed();

    let total: i64 = db_query.clone().count().get_result(&mut conn).unwrap_or(0);
    let results: Vec<Post> = db_query
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

    let post_ids: Vec<i32> = post_tags::table
        .filter(post_tags::tag_id.eq(tag_id))
        .select(post_tags::post_id)
        .load(&mut conn)
        .unwrap_or_default();

    let mut db_query = posts::table
        .filter(posts::id.eq_any(&post_ids))
        .filter(posts::deleted_at.is_null())
        .filter(posts::status.eq("published").or(posts::is_published.eq(true)))
        .into_boxed();

    let total: i64 = db_query.clone().count().get_result(&mut conn).unwrap_or(0);
    let results: Vec<Post> = db_query
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

    let (user_id, status) = match user {
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
        status,
    };

    match diesel::insert_into(comments::table)
        .values(&new_comment)
        .execute(&mut conn)
    {
        Ok(_) => {
            let message = if status == "pending" {
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

pub async fn upload_media(req: HttpRequest, mut payload: actix_multipart::Multipart) -> impl Responder {
    if !is_admin(&req) {
        return HttpResponse::Forbidden().json("Admin access required");
    }

    let user = match get_user_from_request(&req) {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().json("Unauthorized"),
    };

    while let Some(item) = payload.next().await {
        if let Ok(mut field) = item {
            let content_disposition = field.content_disposition();
            let filename = content_disposition
                .get_filename()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "file".to_string());

            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                if let Ok(bytes) = chunk {
                    data.extend_from_slice(&bytes);
                }
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

pub async fn search(query: web::Query<SearchQuery>) -> impl Responder {
    let mut conn = establish_connection();
    let search_pattern = format!("%{}%", query.q);

    let results: Vec<Post> = posts::table
        .filter(posts::deleted_at.is_null())
        .filter(posts::status.eq("published").or(posts::is_published.eq(true)))
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
