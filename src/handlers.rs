use actix_web::{web, HttpResponse, Responder};
use diesel::prelude::*;
use crate::{models::{Post, NewPost, UpdatePost, User, NewUser, Comment, NewComment, UpdateComment, Category, NewCategory, UpdateCategory, Tag, NewTag, UpdateTag, PostTag}, schema::posts, schema::users, schema::comments, schema::categories, schema::tags, schema::post_tags, db::establish_connection, auth::{generate_token, hash_password, verify_password}};

pub async fn get_posts() -> impl Responder {
    let mut conn = establish_connection();
    let results = posts::table.load::<Post>(&mut conn).expect("Error loading posts");
    HttpResponse::Ok().json(results)
}

pub async fn get_post(path: web::Path<i32>) -> impl Responder {
    let id = path.into_inner();
    let mut conn = establish_connection();
    match posts::table.find(id).first::<Post>(&mut conn) {
        Ok(post) => HttpResponse::Ok().json(post),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub async fn create_post(new_post: web::Json<NewPost>) -> impl Responder {
    let mut conn = establish_connection();
    let result = diesel::insert_into(posts::table)
        .values(&*new_post)
        .execute(&mut conn)
        .expect("Error creating post");
    HttpResponse::Created().json(result)
}

pub async fn update_post(path: web::Path<i32>, updated_post: web::Json<UpdatePost>) -> impl Responder {
    let id = path.into_inner();
    let mut conn = establish_connection();
    match diesel::update(posts::table.find(id))
        .set(&*updated_post)
        .execute(&mut conn)
    {
        Ok(affected) if affected > 0 => HttpResponse::Ok().json(affected),
        _ => HttpResponse::NotFound().finish(),
    }
}

pub async fn delete_post(path: web::Path<i32>) -> impl Responder {
    let id = path.into_inner();
    let mut conn = establish_connection();
    match diesel::delete(posts::table.find(id))
        .execute(&mut conn)
    {
        Ok(affected) if affected > 0 => HttpResponse::Ok().json(affected),
        _ => HttpResponse::NotFound().finish(),
    }
}

use validator::Validate;

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

pub async fn register(req: web::Json<RegisterRequest>) -> impl Responder {
    // 验证输入
    if let Err(errors) = req.validate() {
        return HttpResponse::BadRequest().json(errors);
    }
    
    let mut conn = establish_connection();
    
    // 检查用户是否已存在
    if users::table.filter(users::email.eq(&req.email)).first::<User>(&mut conn).is_ok() {
        return HttpResponse::BadRequest().json("Email already exists");
    }
    
    if users::table.filter(users::username.eq(&req.username)).first::<User>(&mut conn).is_ok() {
        return HttpResponse::BadRequest().json("Username already exists");
    }
    
    // 哈希密码
    let password_hash = hash_password(&req.password).unwrap();
    
    // 创建新用户
    let new_user = NewUser {
        username: req.username.clone(),
        email: req.email.clone(),
        password_hash,
        role: "reader".to_string(),
        avatar: None,
        bio: None,
    };
    
    match diesel::insert_into(users::table)
        .values(&new_user)
        .execute(&mut conn)
    {
        Ok(_) => HttpResponse::Created().json("User registered successfully"),
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
    }
}

pub async fn login(req: web::Json<LoginRequest>) -> impl Responder {
    // 验证输入
    if let Err(errors) = req.validate() {
        return HttpResponse::BadRequest().json(errors);
    }
    
    let mut conn = establish_connection();
    
    // 查找用户
    match users::table.filter(users::email.eq(&req.email)).first::<User>(&mut conn) {
        Ok(user) => {
            // 验证密码
            if verify_password(&req.password, &user.password_hash).unwrap() {
                // 生成 JWT token
                let token = generate_token(user.id, &user.username, &user.role).unwrap();
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

pub async fn get_comments(path: web::Path<i32>) -> impl Responder {
    let post_id = path.into_inner();
    let mut conn = establish_connection();
    
    let results = comments::table
        .filter(comments::post_id.eq(post_id))
        .filter(comments::status.eq("approved"))
        .load::<Comment>(&mut conn)
        .expect("Error loading comments");
    
    HttpResponse::Ok().json(results)
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

pub async fn create_comment(req: web::Json<CommentRequest>) -> impl Responder {
    let mut conn = establish_connection();
    
    // 检查文章是否存在
    if posts::table.find(req.post_id).first::<Post>(&mut conn).is_err() {
        return HttpResponse::NotFound().json("Post not found");
    }
    
    // 创建新评论
    let new_comment = NewComment {
        post_id: req.post_id,
        user_id: None, // 暂时只支持游客评论
        parent_id: req.parent_id,
        content: req.content.clone(),
        author_name: req.author_name.clone(),
        author_email: req.author_email.clone(),
        author_website: req.author_website.clone(),
        status: "pending".to_string(), // 默认待审核
    };
    
    match diesel::insert_into(comments::table)
        .values(&new_comment)
        .execute(&mut conn)
    {
        Ok(_) => HttpResponse::Created().json("Comment created successfully, pending approval"),
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
    }
}

pub async fn update_comment(path: web::Path<i32>, req: web::Json<UpdateComment>) -> impl Responder {
    let comment_id = path.into_inner();
    let mut conn = establish_connection();
    
    match diesel::update(comments::table.find(comment_id))
        .set(&*req)
        .execute(&mut conn)
    {
        Ok(affected) if affected > 0 => HttpResponse::Ok().json(affected),
        _ => HttpResponse::NotFound().finish(),
    }
}

pub async fn delete_comment(path: web::Path<i32>) -> impl Responder {
    let comment_id = path.into_inner();
    let mut conn = establish_connection();
    
    match diesel::delete(comments::table.find(comment_id))
        .execute(&mut conn)
    {
        Ok(affected) if affected > 0 => HttpResponse::Ok().json(affected),
        _ => HttpResponse::NotFound().finish(),
    }
}

// 分类相关处理函数
pub async fn get_categories() -> impl Responder {
    let mut conn = establish_connection();
    let results = categories::table.load::<Category>(&mut conn).expect("Error loading categories");
    HttpResponse::Ok().json(results)
}

pub async fn create_category(req: web::Json<NewCategory>) -> impl Responder {
    let mut conn = establish_connection();
    
    // 检查分类是否已存在
    if categories::table.filter(categories::name.eq(&req.name)).first::<Category>(&mut conn).is_ok() {
        return HttpResponse::BadRequest().json("Category already exists");
    }
    
    match diesel::insert_into(categories::table)
        .values(&*req)
        .execute(&mut conn)
    {
        Ok(_) => HttpResponse::Created().json("Category created successfully"),
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
    }
}

pub async fn update_category(path: web::Path<i32>, req: web::Json<UpdateCategory>) -> impl Responder {
    let category_id = path.into_inner();
    let mut conn = establish_connection();
    
    match diesel::update(categories::table.find(category_id))
        .set(&*req)
        .execute(&mut conn)
    {
        Ok(affected) if affected > 0 => HttpResponse::Ok().json(affected),
        _ => HttpResponse::NotFound().finish(),
    }
}

pub async fn delete_category(path: web::Path<i32>) -> impl Responder {
    let category_id = path.into_inner();
    let mut conn = establish_connection();
    
    match diesel::delete(categories::table.find(category_id))
        .execute(&mut conn)
    {
        Ok(affected) if affected > 0 => HttpResponse::Ok().json(affected),
        _ => HttpResponse::NotFound().finish(),
    }
}

// 标签相关处理函数
pub async fn get_tags() -> impl Responder {
    let mut conn = establish_connection();
    let results = tags::table.load::<Tag>(&mut conn).expect("Error loading tags");
    HttpResponse::Ok().json(results)
}

pub async fn create_tag(req: web::Json<NewTag>) -> impl Responder {
    let mut conn = establish_connection();
    
    // 检查标签是否已存在
    if tags::table.filter(tags::name.eq(&req.name)).first::<Tag>(&mut conn).is_ok() {
        return HttpResponse::BadRequest().json("Tag already exists");
    }
    
    match diesel::insert_into(tags::table)
        .values(&*req)
        .execute(&mut conn)
    {
        Ok(_) => HttpResponse::Created().json("Tag created successfully"),
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
    }
}

pub async fn update_tag(path: web::Path<i32>, req: web::Json<UpdateTag>) -> impl Responder {
    let tag_id = path.into_inner();
    let mut conn = establish_connection();
    
    match diesel::update(tags::table.find(tag_id))
        .set(&*req)
        .execute(&mut conn)
    {
        Ok(affected) if affected > 0 => HttpResponse::Ok().json(affected),
        _ => HttpResponse::NotFound().finish(),
    }
}

pub async fn delete_tag(path: web::Path<i32>) -> impl Responder {
    let tag_id = path.into_inner();
    let mut conn = establish_connection();
    
    match diesel::delete(tags::table.find(tag_id))
        .execute(&mut conn)
    {
        Ok(affected) if affected > 0 => HttpResponse::Ok().json(affected),
        _ => HttpResponse::NotFound().finish(),
    }
}

// 文章标签关联处理函数
#[derive(serde::Deserialize)]
pub struct PostTagsRequest {
    pub post_id: i32,
    pub tag_ids: Vec<i32>,
}

pub async fn add_post_tags(req: web::Json<PostTagsRequest>) -> impl Responder {
    let mut conn = establish_connection();
    
    // 检查文章是否存在
    if posts::table.find(req.post_id).first::<Post>(&mut conn).is_err() {
        return HttpResponse::NotFound().json("Post not found");
    }
    
    // 清除现有关联
    diesel::delete(post_tags::table.filter(post_tags::post_id.eq(req.post_id)))
        .execute(&mut conn)
        .expect("Error deleting existing post tags");
    
    // 添加新关联
    for tag_id in &req.tag_ids {
        // 检查标签是否存在
        if tags::table.find(*tag_id).first::<Tag>(&mut conn).is_err() {
            continue;
        }
        
        let post_tag = PostTag {
            post_id: req.post_id,
            tag_id: *tag_id,
        };
        
        diesel::insert_into(post_tags::table)
            .values(&post_tag)
            .execute(&mut conn)
            .expect("Error adding post tag");
    }
    
    HttpResponse::Ok().json("Post tags updated successfully")
}
