use actix_web::{web, HttpResponse, Responder, HttpRequest};
use actix_web::http::header::HeaderMap;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::{models::{Post, NewPost, UpdatePost, User, NewUser, UpdateUser, Comment, NewComment, Category, NewCategory, UpdateCategory, Tag, NewTag, UpdateTag, PostTag, Media, NewMedia}, schema::posts, schema::users, schema::comments, schema::categories, schema::tags, schema::post_tags, schema::media, db::establish_connection, auth::{generate_token, hash_password, verify_password, verify_token, Claims}, roles::{has_permission, can_edit_post, can_delete_post, can_edit_comment, can_delete_comment, PERMISSION_CREATE_POSTS, PERMISSION_EDIT_POSTS, PERMISSION_DELETE_POSTS, PERMISSION_MANAGE_CATEGORIES, PERMISSION_MANAGE_TAGS, PERMISSION_MANAGE_COMMENTS, PERMISSION_MANAGE_MEDIA, PERMISSION_MANAGE_USERS, PERMISSION_MANAGE_ROLES}, email::EmailService};
use validator::Validate;
use chrono::Utc;
use std::sync::Mutex;
use std::collections::HashSet;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, Duration};

// 限流配置结构
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests: u32,    // 最大请求数
    pub window_seconds: u64,  // 时间窗口（秒）
}

// 限流记录结构
#[derive(Debug, Clone)]
pub struct RateLimitRecord {
    pub requests: Vec<SystemTime>,
    pub last_reset: SystemTime,
}

// 缓存项结构
#[derive(Debug, Clone)]
pub struct CacheItem {
    pub data: Vec<u8>,
    pub headers: HeaderMap,
    pub created_at: SystemTime,
    pub ttl: Duration, // 缓存有效期
}

pub struct AppState {
    pub blacklist: Mutex<HashSet<String>>,
    pub view_counter: Mutex<HashMap<String, (i32, SystemTime)>>,
    pub rate_limits: Mutex<HashMap<String, RateLimitRecord>>, // 限流记录，key: "ip:endpoint"
    pub cache: Mutex<HashMap<String, CacheItem>>, // 缓存存储，key: "endpoint:params"
    pub email_service: EmailService,
}



// 不同API端点的默认限流配置
pub fn get_default_rate_limits() -> HashMap<String, RateLimitConfig> {
    let mut configs = HashMap::new();
    
    // 认证相关API - 更严格的限制
    configs.insert("/api/auth/register".to_string(), RateLimitConfig { max_requests: 5, window_seconds: 60 });
    configs.insert("/api/auth/login".to_string(), RateLimitConfig { max_requests: 10, window_seconds: 60 });
    configs.insert("/api/auth/refresh".to_string(), RateLimitConfig { max_requests: 5, window_seconds: 60 });
    
    // 写操作API - 中等限制
    configs.insert("/api/posts".to_string(), RateLimitConfig { max_requests: 20, window_seconds: 60 });
    configs.insert("/api/comments".to_string(), RateLimitConfig { max_requests: 30, window_seconds: 60 });
    configs.insert("/api/categories".to_string(), RateLimitConfig { max_requests: 10, window_seconds: 60 });
    configs.insert("/api/tags".to_string(), RateLimitConfig { max_requests: 10, window_seconds: 60 });
    configs.insert("/api/media".to_string(), RateLimitConfig { max_requests: 15, window_seconds: 60 });
    configs.insert("/api/users".to_string(), RateLimitConfig { max_requests: 10, window_seconds: 60 });
    
    // 读操作API - 较宽松的限制
    configs.insert("/api/posts/hot".to_string(), RateLimitConfig { max_requests: 60, window_seconds: 60 });
    configs.insert("/api/search".to_string(), RateLimitConfig { max_requests: 50, window_seconds: 60 });
    configs.insert("/api/categories/".to_string(), RateLimitConfig { max_requests: 40, window_seconds: 60 });
    configs.insert("/api/tags/".to_string(), RateLimitConfig { max_requests: 40, window_seconds: 60 });
    
    configs
}

// 获取API端点的限流配置
pub fn get_rate_limit_config(endpoint: &str) -> RateLimitConfig {
    let default_configs = get_default_rate_limits();
    
    // 精确匹配
    if let Some(config) = default_configs.get(endpoint) {
        return config.clone();
    }
    
    // 前缀匹配（处理带路径参数的端点）
    for (key, config) in default_configs {
        if endpoint.starts_with(&key) && key.ends_with("/") {
            return config;
        }
    }
    
    // 默认配置
    RateLimitConfig { max_requests: 30, window_seconds: 60 }
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

fn has_permission_from_request(req: &HttpRequest, permission: &str) -> bool {
    get_current_user(req).map(|c| has_permission(&c.role, permission)).unwrap_or(false)
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

#[derive(serde::Deserialize)]
pub struct PostTagsRequest {
    pub post_id: i32,
    pub tag_ids: Vec<i32>,
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

    let search_pattern = query.q.as_ref().map(|q| format!("%{}%", q));
    
    let mut db_query = posts::table
        .filter(posts::deleted_at.is_null())
        .filter(posts::status.eq("published").or(posts::is_published.eq(true)))
        .into_boxed();

    if let Some(category_id) = query.category_id {
        db_query = db_query.filter(posts::category_id.eq(category_id));
    }

    if let Some(ref pattern) = search_pattern {
        db_query = db_query.filter(
            posts::title.like(pattern)
                .or(posts::content.like(pattern))
                .or(posts::excerpt.like(pattern))
        );
    }

    let total: i64 = posts::table
        .filter(posts::deleted_at.is_null())
        .filter(posts::status.eq("published").or(posts::is_published.eq(true)))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);
    
    let results: Vec<Post> = match query.sort_by.as_deref() {
        Some("title") => {
            if query.order.as_deref() == Some("asc") {
                db_query.order(posts::title.asc())
            } else {
                db_query.order(posts::title.desc())
            }
        },
        Some("created_at") => {
            if query.order.as_deref() == Some("asc") {
                db_query.order(posts::created_at.asc())
            } else {
                db_query.order(posts::created_at.desc())
            }
        },
        Some("published_at") => {
            if query.order.as_deref() == Some("asc") {
                db_query.order(posts::published_at.asc())
            } else {
                db_query.order(posts::published_at.desc())
            }
        },
        Some("views") => {
            if query.order.as_deref() == Some("asc") {
                db_query.order(posts::view_count.asc())
            } else {
                db_query.order(posts::view_count.desc())
            }
        },
        _ => db_query.order(posts::created_at.desc()),
    }
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

pub async fn get_post(path: web::Path<i32>, req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let id = path.into_inner();
    let mut conn = establish_connection();
    
    // 获取客户端IP
    let connection_info = req.connection_info();
    let client_ip = connection_info.realip_remote_addr().unwrap_or("unknown");
    let key = format!("{}:{}", client_ip, id);
    
    let now = SystemTime::now();
    let should_increment = {
        let mut view_counter = state.view_counter.lock().unwrap();
        let mut should_increment = false;
        
        if let Some((count, timestamp)) = view_counter.remove(&key) {
            if now.duration_since(timestamp).unwrap_or(Duration::from_secs(61)) < Duration::from_secs(60) {
                if count < 5 {
                    view_counter.insert(key.clone(), (count + 1, now));
                    should_increment = true;
                } else {
                    view_counter.insert(key.clone(), (count, now));
                    should_increment = false;
                }
            } else {
                view_counter.insert(key.clone(), (1, now));
                should_increment = true;
            }
        } else {
            view_counter.insert(key, (1, now));
            should_increment = true;
        }
        
        should_increment
    };
    
    match posts::table.find(id).first::<Post>(&mut conn) {
        Ok(post) => {
            if post.deleted_at.is_some() {
                return HttpResponse::NotFound().finish();
            }
            if post.status.as_deref() != Some("published") && post.is_published != Some(true) {
                return HttpResponse::NotFound().finish();
            }
            
            if should_increment {
                let _ = diesel::update(posts::table.find(id))
                    .set(posts::view_count.eq(post.view_count.unwrap_or(0) + 1))
                    .execute(&mut conn);
            }
            
            HttpResponse::Ok().json(post)
        },
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub async fn get_post_by_slug(path: web::Path<String>, req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
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
            
            // 获取客户端IP
            let connection_info = req.connection_info();
            let client_ip = connection_info.realip_remote_addr().unwrap_or("unknown");
            let key = format!("{}:{}", client_ip, post.id);
            
            let now = SystemTime::now();
            let should_increment = {
                let mut view_counter = state.view_counter.lock().unwrap();
                let mut should_increment = false;
                
                if let Some((count, timestamp)) = view_counter.remove(&key) {
                    if now.duration_since(timestamp).unwrap_or(Duration::from_secs(61)) < Duration::from_secs(60) {
                        if count < 5 {
                            view_counter.insert(key.clone(), (count + 1, now));
                            should_increment = true;
                        } else {
                            view_counter.insert(key.clone(), (count, now));
                            should_increment = false;
                        }
                    } else {
                        view_counter.insert(key.clone(), (1, now));
                        should_increment = true;
                    }
                } else {
                    view_counter.insert(key, (1, now));
                    should_increment = true;
                }
                
                should_increment
            };
            
            if should_increment {
                let _ = diesel::update(posts::table.find(post.id))
                    .set(posts::view_count.eq(post.view_count.unwrap_or(0) + 1))
                    .execute(&mut conn);
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

    if !has_permission_from_request(&req, PERMISSION_CREATE_POSTS) {
        return HttpResponse::Forbidden().json("Permission denied");
    }

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
        is_top: body.is_top,
        allow_comments: body.allow_comments,
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

    if !has_permission_from_request(&req, PERMISSION_EDIT_POSTS) {
        return HttpResponse::Forbidden().json("Permission denied");
    }

    let id = path.into_inner();
    let mut conn = establish_connection();

    match posts::table.find(id).first::<Post>(&mut conn) {
        Ok(post) => {
            if !can_edit_post(&user.role, user.user_id, post.user_id) {
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

    if !has_permission_from_request(&req, PERMISSION_DELETE_POSTS) {
        return HttpResponse::Forbidden().json("Permission denied");
    }

    let id = path.into_inner();
    let mut conn = establish_connection();

    match posts::table.find(id).first::<Post>(&mut conn) {
        Ok(post) => {
            if !can_delete_post(&user.role, user.user_id, post.user_id) {
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

    if !has_permission_from_request(&req, PERMISSION_EDIT_POSTS) {
        return HttpResponse::Forbidden().json("Permission denied");
    }

    let id = path.into_inner();
    let mut conn = establish_connection();
    let app_state = req.app_data::<web::Data<AppState>>().unwrap();

    match posts::table.find(id).first::<Post>(&mut conn) {
        Ok(post) => {
            if !can_edit_post(&user.role, user.user_id, post.user_id) {
                return HttpResponse::Forbidden().json("Cannot update another user's post");
            }

            let old_status = post.status.clone().unwrap_or_else(|| "draft".to_string());
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
                Ok(_) => {
                    // 发送邮件通知
                    if let Some(user_id) = post.user_id {
                        if let Ok(author) = users::table.find(user_id).first::<User>(&mut conn) {
                            let post_url = format!("http://localhost:8080/api/posts/{}", id);
                            let _ = app_state.email_service.send_post_status_change_notification(
                                &author.email,
                                &post.title,
                                &old_status,
                                &body.status,
                                &post_url
                            ).await;
                        }
                    }
                    HttpResponse::Ok().json("Post status updated")
                },
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
    if !has_permission_from_request(&req, PERMISSION_MANAGE_CATEGORIES) {
        return HttpResponse::Forbidden().json("Permission denied");
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
    if !has_permission_from_request(&req, PERMISSION_MANAGE_CATEGORIES) {
        return HttpResponse::Forbidden().json("Permission denied");
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
    if !has_permission_from_request(&req, PERMISSION_MANAGE_CATEGORIES) {
        return HttpResponse::Forbidden().json("Permission denied");
    }

    let id = path.into_inner();
    let mut conn = establish_connection();

    match diesel::delete(categories::table.find(id))
        .execute(&mut conn)
    {
        Ok(affected) if affected > 0 => {

            
            HttpResponse::Ok().json("Category deleted")
        },
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

    let total: i64 = posts::table
        .filter(posts::category_id.eq(category_id))
        .filter(posts::deleted_at.is_null())
        .filter(posts::status.eq("published").or(posts::is_published.eq(true)))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);
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
    if !has_permission_from_request(&req, PERMISSION_MANAGE_TAGS) {
        return HttpResponse::Forbidden().json("Permission denied");
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
    if !has_permission_from_request(&req, PERMISSION_MANAGE_TAGS) {
        return HttpResponse::Forbidden().json("Permission denied");
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
    if !has_permission_from_request(&req, PERMISSION_MANAGE_TAGS) {
        return HttpResponse::Forbidden().json("Permission denied");
    }

    let id = path.into_inner();
    let mut conn = establish_connection();

    match diesel::delete(tags::table.find(id))
        .execute(&mut conn)
    {
        Ok(affected) if affected > 0 => {

            
            HttpResponse::Ok().json("Tag deleted")
        },
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

    let total: i64 = posts::table
        .filter(posts::id.eq_any(&post_ids))
        .filter(posts::deleted_at.is_null())
        .filter(posts::status.eq("published").or(posts::is_published.eq(true)))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);
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
    let app_state = req.app_data::<web::Data<AppState>>().unwrap();

    let mut conn = establish_connection();

    let post = match posts::table.find(body.post_id).first::<Post>(&mut conn) {
        Ok(p) => p,
        Err(_) => return HttpResponse::NotFound().json("Post not found"),
    };

    let (user_id, status) = match user {
        Some(ref u) => (Some(u.user_id), "approved".to_string()),
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
        status: status.clone(),
    };

    match diesel::insert_into(comments::table)
        .values(&new_comment)
        .execute(&mut conn)
    {
        Ok(_) => {
            // 发送邮件通知给文章作者
            if let Some(user_id) = post.user_id {
                if let Ok(author) = users::table.find(user_id).first::<User>(&mut conn) {
                    let comment_author = user.as_ref().map(|u| u.username.clone()).unwrap_or_else(|| body.author_name.clone().unwrap_or_else(|| "Anonymous".to_string()));
                    let post_url = format!("http://localhost:8080/api/posts/{}", body.post_id);
                    let _ = app_state.email_service.send_comment_notification(
                        &author.email,
                        &post.title,
                        &comment_author,
                        &body.content,
                        &post_url
                    ).await;
                }
            }

            let message = if status == "pending" {
                "Comment created successfully, pending approval"
            } else {
                "Comment created successfully"
            };
            HttpResponse::Created().json(serde_json::json!({
                "message": message
            }))
        },
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
    }
}

pub async fn approve_comment(req: HttpRequest, path: web::Path<i32>) -> impl Responder {
    if !has_permission_from_request(&req, PERMISSION_MANAGE_COMMENTS) {
        return HttpResponse::Forbidden().json("Permission denied");
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

    if !has_permission_from_request(&req, PERMISSION_MANAGE_COMMENTS) {
        return HttpResponse::Forbidden().json("Permission denied");
    }

    let comment_id = path.into_inner();
    let mut conn = establish_connection();

    match comments::table.find(comment_id).first::<Comment>(&mut conn) {
        Ok(comment) => {
            if !can_delete_comment(&user.role, user.user_id, comment.user_id) {
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

pub async fn get_comment_replies(path: web::Path<i32>) -> impl Responder {
    let comment_id = path.into_inner();
    let mut conn = establish_connection();

    let results = comments::table
        .filter(comments::parent_id.eq(comment_id))
        .filter(comments::status.eq("approved"))
        .order(comments::created_at.desc())
        .load::<Comment>(&mut conn)
        .unwrap_or_default();

    HttpResponse::Ok().json(results)
}

pub async fn upload_media(req: HttpRequest, _payload: actix_multipart::Multipart) -> impl Responder {
    if !has_permission_from_request(&req, PERMISSION_MANAGE_MEDIA) {
        return HttpResponse::Forbidden().json("Permission denied");
    }

    let _user = match get_user_from_request(&req) {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().json("Unauthorized"),
    };

    // TODO: Fix multipart handling
    HttpResponse::Ok().json("Media upload not implemented yet")
}

pub async fn get_media(req: HttpRequest) -> impl Responder {
    if !has_permission_from_request(&req, PERMISSION_MANAGE_MEDIA) {
        return HttpResponse::Forbidden().json("Permission denied");
    }

    let mut conn = establish_connection();
    let results = media::table.load::<Media>(&mut conn).unwrap_or_default();
    HttpResponse::Ok().json(results)
}

pub async fn delete_media(req: HttpRequest, path: web::Path<i32>) -> impl Responder {
    if !has_permission_from_request(&req, PERMISSION_MANAGE_MEDIA) {
        return HttpResponse::Forbidden().json("Permission denied");
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

pub async fn get_hot_posts() -> impl Responder {
    let mut conn = establish_connection();
    
    let results: Vec<Post> = posts::table
        .filter(posts::deleted_at.is_null())
        .filter(posts::status.eq("published").or(posts::is_published.eq(true)))
        .order(posts::view_count.desc())
        .limit(10)
        .load(&mut conn)
        .unwrap_or_default();

    HttpResponse::Ok().json(results)
}

pub async fn add_post_tags(req: HttpRequest, body: web::Json<PostTagsRequest>) -> impl Responder {
    let user = match get_user_from_request(&req) {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().json("Unauthorized"),
    };

    if !has_permission_from_request(&req, PERMISSION_EDIT_POSTS) {
        return HttpResponse::Forbidden().json("Permission denied");
    }

    let mut conn = establish_connection();

    match posts::table.find(body.post_id).first::<Post>(&mut conn) {
        Ok(post) => {
            if !can_edit_post(&user.role, user.user_id, post.user_id) {
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

// 角色管理API
pub async fn get_roles() -> impl Responder {
    use crate::roles::{get_role_permissions, ROLE_ADMIN, ROLE_EDITOR, ROLE_AUTHOR, ROLE_SUBSCRIBER};
    
    let roles = vec![
        serde_json::json!({
            "name": ROLE_ADMIN,
            "display_name": "管理员",
            "permissions": get_role_permissions().get(ROLE_ADMIN).unwrap()
        }),
        serde_json::json!({
            "name": ROLE_EDITOR,
            "display_name": "编辑",
            "permissions": get_role_permissions().get(ROLE_EDITOR).unwrap()
        }),
        serde_json::json!({
            "name": ROLE_AUTHOR,
            "display_name": "作者",
            "permissions": get_role_permissions().get(ROLE_AUTHOR).unwrap()
        }),
        serde_json::json!({
            "name": ROLE_SUBSCRIBER,
            "display_name": "订阅者",
            "permissions": get_role_permissions().get(ROLE_SUBSCRIBER).unwrap()
        }),
    ];
    
    HttpResponse::Ok().json(roles)
}

pub async fn get_role(path: web::Path<String>) -> impl Responder {
    use crate::roles::{get_role_permissions};
    
    let role_name = path.into_inner();
    let role_permissions = get_role_permissions();
    
    if let Some(permissions) = role_permissions.get(role_name.as_str()) {
        let display_name = match role_name.as_str() {
            "admin" => "管理员",
            "editor" => "编辑",
            "author" => "作者",
            "subscriber" => "订阅者",
            _ => &role_name,
        };
        
        HttpResponse::Ok().json(serde_json::json!({
            "name": role_name,
            "display_name": display_name,
            "permissions": permissions
        }))
    } else {
        HttpResponse::NotFound().json("Role not found")
    }
}

// 用户管理API
pub async fn get_users(req: HttpRequest) -> impl Responder {
    if !has_permission_from_request(&req, PERMISSION_MANAGE_USERS) {
        return HttpResponse::Forbidden().json("Permission denied");
    }

    let mut conn = establish_connection();
    let results = users::table.load::<User>(&mut conn).unwrap_or_default();
    HttpResponse::Ok().json(results)
}

pub async fn get_user(req: HttpRequest, path: web::Path<i32>) -> impl Responder {
    if !has_permission_from_request(&req, PERMISSION_MANAGE_USERS) {
        return HttpResponse::Forbidden().json("Permission denied");
    }

    let id = path.into_inner();
    let mut conn = establish_connection();
    match users::table.find(id).first::<User>(&mut conn) {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub async fn create_user(req: HttpRequest, body: web::Json<NewUser>) -> impl Responder {
    if !has_permission_from_request(&req, PERMISSION_MANAGE_USERS) {
        return HttpResponse::Forbidden().json("Permission denied");
    }

    let mut conn = establish_connection();

    if users::table.filter(users::email.eq(&body.email)).first::<User>(&mut conn).is_ok() {
        return HttpResponse::BadRequest().json("Email already exists");
    }

    if users::table.filter(users::username.eq(&body.username)).first::<User>(&mut conn).is_ok() {
        return HttpResponse::BadRequest().json("Username already exists");
    }

    match diesel::insert_into(users::table)
        .values(&*body)
        .execute(&mut conn)
    {
        Ok(_) => {
            let created: User = users::table.order(users::id.desc()).first(&mut conn).unwrap();
            HttpResponse::Created().json(created)
        },
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
    }
}

pub async fn update_user(req: HttpRequest, path: web::Path<i32>, body: web::Json<UpdateUser>) -> impl Responder {
    if !has_permission_from_request(&req, PERMISSION_MANAGE_USERS) {
        return HttpResponse::Forbidden().json("Permission denied");
    }

    let id = path.into_inner();
    let mut conn = establish_connection();

    match diesel::update(users::table.find(id))
        .set(&*body)
        .execute(&mut conn)
    {
        Ok(affected) if affected > 0 => HttpResponse::Ok().json("User updated"),
        _ => HttpResponse::NotFound().finish(),
    }
}

pub async fn delete_user(req: HttpRequest, path: web::Path<i32>) -> impl Responder {
    if !has_permission_from_request(&req, PERMISSION_MANAGE_USERS) {
        return HttpResponse::Forbidden().json("Permission denied");
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

pub async fn send_test_email(req: HttpRequest, body: web::Json<TestEmailRequest>) -> impl Responder {
    if !has_permission_from_request(&req, PERMISSION_MANAGE_USERS) {
        return HttpResponse::Forbidden().json("Permission denied");
    }

    let app_state = req.app_data::<web::Data<AppState>>().unwrap();
    match app_state.email_service.send_test_email(&body.email).await {
        Ok(_) => HttpResponse::Ok().json("Test email sent successfully"),
        Err(e) => HttpResponse::InternalServerError().json(format!("Error sending test email: {}", e)),
    }
}

#[derive(Deserialize)]
pub struct TestEmailRequest {
    pub email: String,
}
