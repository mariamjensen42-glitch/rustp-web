use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use actix_files::Files;
use std::sync::Mutex;
use std::collections::HashSet;
use tokio::time::{interval, Duration};
use chrono::Utc;
use diesel::prelude::*;
use crate::handlers::{
    get_posts, get_post, get_post_by_slug, create_post, update_post, delete_post, update_post_status,
    register, login, logout, refresh_token, get_me, update_me, update_password,
    get_comments, create_comment, approve_comment, delete_comment,
    get_categories, get_category, create_category, update_category, delete_category, get_category_posts,
    get_tags, create_tag, update_tag, delete_tag, get_tag_posts,
    upload_media, get_media, delete_media,
    search, add_post_tags as add_post_tags_handler, AppState,
    get_post_versions, get_post_version, rollback_to_version,
    schedule_post, get_post_analytics, get_related_posts, save_draft, get_drafts,
    upload_avatar,
    get_post_recommendations, get_my_read_history, delete_read_history_item, clear_read_history,
};
use crate::db::establish_connection;
use crate::schema::posts;

mod models;
mod schema;
mod db;
mod handlers;
mod auth;

async fn check_scheduled_posts() {
    let mut interval = interval(Duration::from_secs(60));
    println!("Scheduled post checker started, checking every 60 seconds");
    
    loop {
        interval.tick().await;
        
        let mut conn = establish_connection();
        let now = Utc::now().naive_utc();
        
        // 查找需要发布的文章
        let posts_to_publish: Vec<i32> = posts::table
            .filter(posts::is_scheduled.eq(true))
            .filter(posts::scheduled_at.le(now))
            .filter(posts::deleted_at.is_null())
            .select(posts::id)
            .load(&mut conn)
            .unwrap_or_default();
        
        if !posts_to_publish.is_empty() {
            println!("Publishing {} scheduled posts", posts_to_publish.len());
            
            // 更新文章状态为已发布
            for post_id in posts_to_publish {
                let _ = diesel::update(posts::table.find(post_id))
                    .set((
                        posts::status.eq("published"),
                        posts::is_published.eq(true),
                        posts::is_scheduled.eq(false),
                        posts::published_at.eq(now),
                        posts::updated_at.eq(now),
                    ))
                    .execute(&mut conn);
            }
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        blacklist: Mutex::new(HashSet::new()),
    });

    // 启动后台调度任务
    tokio::spawn(check_scheduled_posts());

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(app_state.clone())
            .wrap(cors)
            .route("/api/auth/register", web::post().to(register))
            .route("/api/auth/login", web::post().to(login))
            .route("/api/auth/logout", web::post().to(logout))
            .route("/api/auth/refresh", web::post().to(refresh_token))
            .route("/api/users/me", web::get().to(get_me))
            .route("/api/users/me", web::put().to(update_me))
            .route("/api/users/me/password", web::put().to(update_password))
            .route("/api/posts", web::get().to(get_posts))
            .route("/api/posts", web::post().to(create_post))
            .route("/api/posts/{id}", web::get().to(get_post))
            .route("/api/posts/slug/{slug}", web::get().to(get_post_by_slug))
            .route("/api/posts/{id}", web::put().to(update_post))
            .route("/api/posts/{id}", web::delete().to(delete_post))
            .route("/api/posts/{id}/status", web::patch().to(update_post_status))
            .route("/api/posts/{id}/comments", web::get().to(get_comments))
            .route("/api/posts/{id}/comments", web::post().to(create_comment))
            .route("/api/posts/tags", web::post().to(add_post_tags_handler))
            .route("/api/comments/{id}/approve", web::put().to(approve_comment))
            .route("/api/comments/{id}", web::delete().to(delete_comment))
            .route("/api/categories", web::get().to(get_categories))
            .route("/api/categories", web::post().to(create_category))
            .route("/api/categories/{id}", web::get().to(get_category))
            .route("/api/categories/{id}", web::put().to(update_category))
            .route("/api/categories/{id}", web::delete().to(delete_category))
            .route("/api/categories/{id}/posts", web::get().to(get_category_posts))
            .route("/api/tags", web::get().to(get_tags))
            .route("/api/tags", web::post().to(create_tag))
            .route("/api/tags/{id}", web::put().to(update_tag))
            .route("/api/tags/{id}", web::delete().to(delete_tag))
            .route("/api/tags/{id}/posts", web::get().to(get_tag_posts))
            .route("/api/media/upload", web::post().to(upload_media))
            .route("/api/media", web::get().to(get_media))
            .route("/api/media/{id}", web::delete().to(delete_media))
            .route("/api/search", web::get().to(search))
            .route("/api/posts/{id}/versions", web::get().to(get_post_versions))
            .route("/api/posts/{id}/versions/{version_number}", web::get().to(get_post_version))
            .route("/api/posts/{id}/rollback", web::post().to(rollback_to_version))
            .route("/api/posts/{id}/schedule", web::post().to(schedule_post))
            .route("/api/posts/{id}/analytics", web::get().to(get_post_analytics))
            .route("/api/posts/{id}/related", web::get().to(get_related_posts))
            .route("/api/posts/{id}/draft", web::post().to(save_draft))
            .route("/api/posts/drafts", web::get().to(get_drafts))
            .route("/api/users/me/avatar", web::post().to(upload_avatar))
            .route("/api/posts/{id}/recommendations", web::get().to(get_post_recommendations))
            .route("/api/users/me/read-history", web::get().to(get_my_read_history))
            .route("/api/users/me/read-history/{post_id}", web::delete().to(delete_read_history_item))
            .route("/api/users/me/read-history", web::delete().to(clear_read_history))
            .service(Files::new("/uploads", "./uploads").show_files_listing())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
