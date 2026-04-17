use actix_web::{web, HttpResponse, Responder, HttpRequest, HttpMessage};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::{models::{Post, NewPost, UpdatePost, User, NewUser, Comment, NewComment, UpdateComment, Category, NewCategory, UpdateCategory, Tag, NewTag, UpdateTag, PostTag, Media, NewMedia, PostVersion, NewPostVersion, PostAnalytic, NewPostAnalytic, UpdatePostAnalytic}, schema::posts, schema::users, schema::comments, schema::categories, schema::tags, schema::post_tags, schema::media, schema::post_versions, schema::post_analytics, db::establish_connection, auth::{generate_token, hash_password, verify_password, verify_token, Claims}};
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
    let now = Utc::now().naive_utc();

    let mut total_query = posts::table
        .filter(posts::deleted_at.is_null())
        .filter(
            (posts::status.eq("published").or(posts::is_published.eq(true)))
            .or(
                posts::is_scheduled.eq(true)
                    .and(posts::scheduled_at.le(now))
            )
        )
        .into_boxed();

    let mut data_query = posts::table
        .filter(posts::deleted_at.is_null())
        .filter(
            (posts::status.eq("published").or(posts::is_published.eq(true)))
            .or(
                posts::is_scheduled.eq(true)
                    .and(posts::scheduled_at.le(now))
            )
        )
        .into_boxed();

    if let Some(category_id) = query.category_id {
        total_query = total_query.filter(posts::category_id.eq(category_id));
        data_query = data_query.filter(posts::category_id.eq(category_id));
    }

    if let Some(q) = &query.q {
        let search_pattern = format!("%{}%", q);
        total_query = total_query.filter(
            posts::title.like(search_pattern.clone())
                .or(posts::content.like(search_pattern.clone()))
                .or(posts::excerpt.like(search_pattern))
        );
        let search_pattern2 = format!("%{}%", q);
        data_query = data_query.filter(
            posts::title.like(search_pattern2.clone())
                .or(posts::content.like(search_pattern2.clone()))
                .or(posts::excerpt.like(search_pattern2))
        );
    }

    let total: i64 = total_query.count().get_result(&mut conn).unwrap_or(0);
    
    let results: Vec<Post> = data_query
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

pub async fn get_post(path: web::Path<i32>) -> impl Responder {
    let id = path.into_inner();
    let mut conn = establish_connection();
    let now = Utc::now().naive_utc();

    match posts::table.find(id).first::<Post>(&mut conn) {
        Ok(post) => {
            if post.deleted_at.is_some() {
                return HttpResponse::NotFound().finish();
            }
            
            let is_published = post.status.as_deref() == Some("published") || post.is_published == Some(true);
            let is_scheduled_and_ready = post.is_scheduled == Some(true) 
                && post.scheduled_at.map(|dt| dt <= now).unwrap_or(false);
            
            if !is_published && !is_scheduled_and_ready {
                return HttpResponse::NotFound().finish();
            }

            let today = Utc::now().naive_utc().date();
            
            let existing_analytic = post_analytics::table
                .filter(post_analytics::post_id.eq(id))
                .filter(post_analytics::visit_date.eq(today))
                .first::<PostAnalytic>(&mut conn);

            match existing_analytic {
                Ok(analytic) => {
                    let update_data = UpdatePostAnalytic {
                        visit_count: Some(analytic.visit_count + 1),
                        unique_visitors: Some(analytic.unique_visitors),
                        updated_at: Some(Utc::now().naive_utc()),
                    };
                    let _ = diesel::update(post_analytics::table.find(analytic.id))
                        .set(&update_data)
                        .execute(&mut conn);
                },
                Err(_) => {
                    let new_analytic = NewPostAnalytic {
                        post_id: id,
                        visit_date: today,
                        visit_count: 1,
                        unique_visitors: 1,
                    };
                    let _ = diesel::insert_into(post_analytics::table)
                        .values(&new_analytic)
                        .execute(&mut conn);
                }
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
        is_top: body.is_top,
        allow_comments: body.allow_comments,
        scheduled_at: None,
        is_scheduled: None,
        auto_save_draft: Some(true),
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

            let latest_version = post_versions::table
                .filter(post_versions::post_id.eq(id))
                .select(diesel::dsl::max(post_versions::version_number))
                .first::<Option<i32>>(&mut conn)
                .unwrap_or(Some(0))
                .unwrap_or(0);

            let new_version = NewPostVersion {
                post_id: id,
                version_number: latest_version + 1,
                title: post.title,
                content: post.content,
                excerpt: post.excerpt,
                summary: post.summary,
                cover_image: post.cover_image,
                created_by: Some(user.user_id),
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
                updated_at: Some(Utc::now().naive_utc()),
                ..Default::default()
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

    let total_query = posts::table
        .filter(posts::category_id.eq(category_id))
        .filter(posts::deleted_at.is_null())
        .filter(posts::status.eq("published").or(posts::is_published.eq(true)));

    let data_query = posts::table
        .filter(posts::category_id.eq(category_id))
        .filter(posts::deleted_at.is_null())
        .filter(posts::status.eq("published").or(posts::is_published.eq(true)));

    let total: i64 = total_query.count().get_result(&mut conn).unwrap_or(0);
    let results: Vec<Post> = data_query
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

    let total_query = posts::table
        .filter(posts::id.eq_any(&post_ids))
        .filter(posts::deleted_at.is_null())
        .filter(posts::status.eq("published").or(posts::is_published.eq(true)));

    let data_query = posts::table
        .filter(posts::id.eq_any(&post_ids))
        .filter(posts::deleted_at.is_null())
        .filter(posts::status.eq("published").or(posts::is_published.eq(true)));

    let total: i64 = total_query.count().get_result(&mut conn).unwrap_or(0);
    let results: Vec<Post> = data_query
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

    let (user_id, status_val) = match user {
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
        status: status_val.clone(),
    };

    match diesel::insert_into(comments::table)
        .values(&new_comment)
        .execute(&mut conn)
    {
        Ok(_) => {
            let message = if status_val == "pending" {
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

// pub async fn upload_media(req: HttpRequest, mut payload: actix_multipart::Multipart) -> impl Responder {
//     if !is_admin(&req) {
//         return HttpResponse::Forbidden().json("Admin access required");
//     }

//     let user = match get_user_from_request(&req) {
//         Some(u) => u,
//         None => return HttpResponse::Unauthorized().json("Unauthorized"),
//     };

//     while let Some(item) = payload.next().await {
//         if let Ok(mut field) = item {
//             let content_disposition = field.content_disposition();
//             let filename = content_disposition
//                 .get_filename()
//                 .map(|s| s.to_string())
//                 .unwrap_or_else(|| "file".to_string());

//             let mut data = Vec::new();
//             while let Some(chunk) = field.next().await {
//                 if let Ok(bytes) = chunk {
//                     data.extend_from_slice(&bytes);
//                 }
//             }

//             let filepath = format!("./uploads/{}", filename);
//             let mimetype = field
//                 .content_type()
//                 .map(|m| m.to_string())
//                 .unwrap_or_else(|| "application/octet-stream".to_string());

//             let mut conn = establish_connection();
//             let new_media = NewMedia {
//                 filename: filename.clone(),
//                 filepath: filepath.clone(),
//                 mimetype,
//                 size: data.len() as i64,
//                 uploaded_by: Some(user.user_id),
//             };

//             match diesel::insert_into(media::table)
//                 .values(&new_media)
//                 .execute(&mut conn)
//             {
//                 Ok(_) => {
//                     if let Ok(_) = std::fs::create_dir_all("./uploads") {
//                         let _ = std::fs::write(&filepath, &data);
//                     }
//                     return HttpResponse::Created().json(serde_json::json!({
//                         "filename": filename,
//                         "filepath": filepath
//                     }));
//                 },
//                 Err(e) => return HttpResponse::InternalServerError().json(format!("Error: {}", e)),
//             }
//         }
//     }

//     HttpResponse::BadRequest().json("No file uploaded")
// }

// pub async fn get_media(req: HttpRequest) -> impl Responder {
//     if !is_admin(&req) {
//         return HttpResponse::Forbidden().json("Admin access required");
//     }

//     let mut conn = establish_connection();
//     let results = media::table.load::<Media>(&mut conn).unwrap_or_default();
//     HttpResponse::Ok().json(results)
// }

// pub async fn delete_media(req: HttpRequest, path: web::Path<i32>) -> impl Responder {
//     if !is_admin(&req) {
//         return HttpResponse::Forbidden().json("Admin access required");
//     }

//     let id = path.into_inner();
//     let mut conn = establish_connection();

//     match diesel::delete(media::table.find(id))
//         .execute(&mut conn)
//     {
//         Ok(_) => HttpResponse::Ok().json("Media deleted"),
//         Err(_) => HttpResponse::NotFound().finish(),
//     }
// }

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

#[derive(serde::Deserialize)]
pub struct SchedulePostRequest {
    pub scheduled_at: Option<chrono::NaiveDateTime>,
}

#[derive(serde::Deserialize)]
pub struct RollbackPostRequest {
    pub version_number: i32,
}

#[derive(serde::Deserialize)]
pub struct SaveDraftRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub excerpt: Option<String>,
    pub summary: Option<String>,
}

pub async fn get_post_versions(req: HttpRequest, path: web::Path<i32>) -> impl Responder {
    let user = match get_user_from_request(&req) {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().json("Unauthorized"),
    };

    let post_id = path.into_inner();
    let mut conn = establish_connection();

    match posts::table.find(post_id).first::<Post>(&mut conn) {
        Ok(post) => {
            if post.user_id != Some(user.user_id) && user.role != "admin" {
                return HttpResponse::Forbidden().json("Cannot access another user's post versions");
            }

            let versions: Vec<PostVersion> = post_versions::table
                .filter(post_versions::post_id.eq(post_id))
                .order(post_versions::version_number.desc())
                .load(&mut conn)
                .unwrap_or_default();

            HttpResponse::Ok().json(versions)
        },
        Err(_) => HttpResponse::NotFound().json("Post not found"),
    }
}

pub async fn get_post_version(req: HttpRequest, path: web::Path<(i32, i32)>) -> impl Responder {
    let user = match get_user_from_request(&req) {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().json("Unauthorized"),
    };

    let (post_id, version_number) = path.into_inner();
    let mut conn = establish_connection();

    match posts::table.find(post_id).first::<Post>(&mut conn) {
        Ok(post) => {
            if post.user_id != Some(user.user_id) && user.role != "admin" {
                return HttpResponse::Forbidden().json("Cannot access another user's post version");
            }

            match post_versions::table
                .filter(post_versions::post_id.eq(post_id))
                .filter(post_versions::version_number.eq(version_number))
                .first::<PostVersion>(&mut conn)
            {
                Ok(version) => HttpResponse::Ok().json(version),
                Err(_) => HttpResponse::NotFound().json("Version not found"),
            }
        },
        Err(_) => HttpResponse::NotFound().json("Post not found"),
    }
}

pub async fn rollback_to_version(req: HttpRequest, path: web::Path<i32>, body: web::Json<RollbackPostRequest>) -> impl Responder {
    let user = match get_user_from_request(&req) {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().json("Unauthorized"),
    };

    let post_id = path.into_inner();
    let mut conn = establish_connection();

    match posts::table.find(post_id).first::<Post>(&mut conn) {
        Ok(post) => {
            if post.user_id != Some(user.user_id) && user.role != "admin" {
                return HttpResponse::Forbidden().json("Cannot rollback another user's post");
            }

            match post_versions::table
                .filter(post_versions::post_id.eq(post_id))
                .filter(post_versions::version_number.eq(body.version_number))
                .first::<PostVersion>(&mut conn)
            {
                Ok(version) => {
                    let latest_version = post_versions::table
                        .filter(post_versions::post_id.eq(post_id))
                        .select(diesel::dsl::max(post_versions::version_number))
                        .first::<Option<i32>>(&mut conn)
                        .unwrap_or(Some(0))
                        .unwrap_or(0);

                    let new_version = NewPostVersion {
                        post_id,
                        version_number: latest_version + 1,
                        title: post.title,
                        content: post.content,
                        excerpt: post.excerpt,
                        summary: post.summary,
                        cover_image: post.cover_image,
                        created_by: Some(user.user_id),
                    };

                    let _ = diesel::insert_into(post_versions::table)
                        .values(&new_version)
                        .execute(&mut conn);

                    let update_data = UpdatePost {
                        title: Some(version.title),
                        content: Some(version.content),
                        excerpt: version.excerpt,
                        summary: version.summary,
                        cover_image: version.cover_image,
                        updated_at: Some(Utc::now().naive_utc()),
                        ..Default::default()
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
        Err(_) => HttpResponse::NotFound().json("Post not found"),
    }
}

pub async fn schedule_post(req: HttpRequest, path: web::Path<i32>, body: web::Json<SchedulePostRequest>) -> impl Responder {
    let user = match get_user_from_request(&req) {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().json("Unauthorized"),
    };

    let post_id = path.into_inner();
    let mut conn = establish_connection();

    match posts::table.find(post_id).first::<Post>(&mut conn) {
        Ok(post) => {
            if post.user_id != Some(user.user_id) && user.role != "admin" {
                return HttpResponse::Forbidden().json("Cannot schedule another user's post");
            }

            let (is_scheduled, scheduled_at) = match &body.scheduled_at {
                Some(dt) => (Some(true), Some(*dt)),
                None => (Some(false), None),
            };

            let update_data = UpdatePost {
                scheduled_at,
                is_scheduled,
                status: if is_scheduled == Some(true) { Some("scheduled".to_string()) } else { post.status },
                ..Default::default()
            };

            match diesel::update(posts::table.find(post_id))
                .set(&update_data)
                .execute(&mut conn)
            {
                Ok(_) => HttpResponse::Ok().json("Post scheduled successfully"),
                Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
            }
        },
        Err(_) => HttpResponse::NotFound().json("Post not found"),
    }
}

pub async fn get_post_analytics(req: HttpRequest, path: web::Path<i32>) -> impl Responder {
    let user = match get_user_from_request(&req) {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().json("Unauthorized"),
    };

    let post_id = path.into_inner();
    let mut conn = establish_connection();

    match posts::table.find(post_id).first::<Post>(&mut conn) {
        Ok(post) => {
            if post.user_id != Some(user.user_id) && user.role != "admin" {
                return HttpResponse::Forbidden().json("Cannot access another user's post analytics");
            }

            let analytics: Vec<PostAnalytic> = post_analytics::table
                .filter(post_analytics::post_id.eq(post_id))
                .order(post_analytics::visit_date.desc())
                .load(&mut conn)
                .unwrap_or_default();

            let total_visits: i64 = analytics.iter().map(|a| a.visit_count as i64).sum();
            let total_unique_visitors: i64 = analytics.iter().map(|a| a.unique_visitors as i64).sum();

            HttpResponse::Ok().json(serde_json::json!({
                "daily_analytics": analytics,
                "total_visits": total_visits,
                "total_unique_visitors": total_unique_visitors,
                "current_view_count": post.view_count.unwrap_or(0)
            }))
        },
        Err(_) => HttpResponse::NotFound().json("Post not found"),
    }
}

pub async fn get_related_posts(path: web::Path<i32>) -> impl Responder {
    let post_id = path.into_inner();
    let mut conn = establish_connection();

    match posts::table.find(post_id).first::<Post>(&mut conn) {
        Ok(post) => {
            let mut related_posts = Vec::new();

            if let Some(category_id) = post.category_id {
                let category_posts: Vec<Post> = posts::table
                    .filter(posts::category_id.eq(category_id))
                    .filter(posts::id.ne(post_id))
                    .filter(posts::deleted_at.is_null())
                    .filter(posts::status.eq("published").or(posts::is_published.eq(true)))
                    .order(posts::view_count.desc())
                    .limit(5)
                    .load(&mut conn)
                    .unwrap_or_default();
                related_posts.extend(category_posts);
            }

            let tag_ids: Vec<i32> = post_tags::table
                .filter(post_tags::post_id.eq(post_id))
                .select(post_tags::tag_id)
                .load(&mut conn)
                .unwrap_or_default();

            if !tag_ids.is_empty() {
                let tag_post_ids: Vec<i32> = post_tags::table
                    .filter(post_tags::tag_id.eq_any(&tag_ids))
                    .filter(post_tags::post_id.ne(post_id))
                    .select(post_tags::post_id)
                    .distinct()
                    .load(&mut conn)
                    .unwrap_or_default();

                let tag_posts: Vec<Post> = posts::table
                    .filter(posts::id.eq_any(&tag_post_ids))
                    .filter(posts::id.ne(post_id))
                    .filter(posts::deleted_at.is_null())
                    .filter(posts::status.eq("published").or(posts::is_published.eq(true)))
                    .order(posts::view_count.desc())
                    .limit(5)
                    .load(&mut conn)
                    .unwrap_or_default();
                related_posts.extend(tag_posts);
            }

            let mut seen_ids = HashSet::new();
            let unique_posts: Vec<Post> = related_posts
                .into_iter()
                .filter(|p| seen_ids.insert(p.id))
                .take(10)
                .collect();

            HttpResponse::Ok().json(unique_posts)
        },
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub async fn save_draft(req: HttpRequest, path: web::Path<i32>, body: web::Json<SaveDraftRequest>) -> impl Responder {
    let user = match get_user_from_request(&req) {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().json("Unauthorized"),
    };

    let post_id = path.into_inner();
    let mut conn = establish_connection();

    match posts::table.find(post_id).first::<Post>(&mut conn) {
        Ok(post) => {
            if post.user_id != Some(user.user_id) && user.role != "admin" {
                return HttpResponse::Forbidden().json("Cannot edit another user's post");
            }

            let update_data = UpdatePost {
                title: body.title.clone(),
                content: body.content.clone(),
                excerpt: body.excerpt.clone(),
                summary: body.summary.clone(),
                draft_saved_at: Some(Utc::now().naive_utc()),
                status: Some("draft".to_string()),
                ..Default::default()
            };

            match diesel::update(posts::table.find(post_id))
                .set(&update_data)
                .execute(&mut conn)
            {
                Ok(_) => HttpResponse::Ok().json("Draft saved successfully"),
                Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
            }
        },
        Err(_) => HttpResponse::NotFound().json("Post not found"),
    }
}

pub async fn get_drafts(req: HttpRequest, query: web::Query<PaginationQuery>) -> impl Responder {
    let user = match get_user_from_request(&req) {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().json("Unauthorized"),
    };

    let mut conn = establish_connection();
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(10).min(100);
    let offset = (page - 1) * per_page;

    let mut total_query = posts::table
        .filter(posts::deleted_at.is_null())
        .filter(posts::status.eq("draft"))
        .into_boxed();

    let mut data_query = posts::table
        .filter(posts::deleted_at.is_null())
        .filter(posts::status.eq("draft"))
        .into_boxed();

    if user.role != "admin" {
        total_query = total_query.filter(posts::user_id.eq(user.user_id));
        data_query = data_query.filter(posts::user_id.eq(user.user_id));
    }

    let total: i64 = total_query.count().get_result(&mut conn).unwrap_or(0);
    let results: Vec<Post> = data_query
        .order(posts::draft_saved_at.desc())
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
