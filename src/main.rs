use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use std::sync::Mutex;
use std::collections::HashSet;
use tokio::time;
use chrono::Utc;
use diesel::prelude::*;
use crate::db::establish_connection;
use crate::schema::posts::dsl::posts;
use crate::handlers::{AppState,
    get_posts, get_post, get_post_by_slug, create_post, update_post, delete_post, update_post_status,
    register, login, logout, refresh_token, get_me, update_me, update_password,
    get_comments, create_comment, approve_comment, delete_comment,
    get_categories, get_category, create_category, update_category, delete_category, get_category_posts,
    get_tags, create_tag, update_tag, delete_tag, get_tag_posts, add_post_tags,
    upload_media, get_media, delete_media, get_media_categories, search_media, batch_update_media, batch_delete_media,
    search, add_post_tags as add_post_tags_handler, get_admin_users, update_admin_user, delete_admin_user,
    get_admin_comments, batch_approve_comments, batch_delete_comments, get_stats_overview, get_stats_visits, get_stats_users, get_post_versions, rollback_post_version,
    generate_sitemap, generate_robots, update_post_seo_metadata};
use crate::security::{SecurityState, SecurityMiddleware, get_security_logs};
use crate::cache::{CacheState, CacheMiddleware};
use crate::monitoring::{MonitoringState, MonitoringMiddleware};

mod models;
mod schema;
mod db;
mod handlers;
mod auth;
mod security;
mod cache;
mod monitoring;



// 定时检查并发布达到发布时间的文章
async fn check_publish_scheduled_posts() {
    loop {
        time::sleep(time::Duration::from_secs(60)).await; // 每分钟检查一次
        
        let mut conn = establish_connection();
        let now = Utc::now().naive_utc();
        
        // 查找所有达到发布时间但尚未发布的文章
        let posts_to_publish = posts
            .filter(crate::schema::posts::published_at.is_not_null())
            .filter(crate::schema::posts::published_at.le(now))
            .filter(crate::schema::posts::status.ne("published"))
            .filter(crate::schema::posts::is_published.ne(true))
            .load::<crate::models::Post>(&mut conn)
            .unwrap_or_default();
        
        // 更新这些文章的状态为已发布
        for post in posts_to_publish {
            diesel::update(posts.find(post.id))
                .set((
                    crate::schema::posts::status.eq("published"),
                    crate::schema::posts::is_published.eq(true),
                ))
                .execute(&mut conn)
                .ok();
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 启动定时任务
    tokio::spawn(check_publish_scheduled_posts());
    
    let app_state = web::Data::new(AppState {
        blacklist: Mutex::new(HashSet::new()),
    });

    let security_state = web::Data::new(SecurityState::new());
    let cache_state = web::Data::new(CacheState::new());
    let monitoring_state = web::Data::new(MonitoringState::new());

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(app_state.clone())
            .app_data(security_state.clone())
            .app_data(cache_state.clone())
            .app_data(monitoring_state.clone())
            .wrap(cors)
            .wrap(actix_web::middleware::Logger::default())
            .wrap(MonitoringMiddleware)
            .wrap(SecurityMiddleware)
            .wrap(CacheMiddleware)
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
            .route("/api/posts/{id}/versions", web::get().to(get_post_versions))
            .route("/api/posts/{id}/versions/{version_id}/rollback", web::post().to(rollback_post_version))
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
            .route("/api/media/categories", web::get().to(get_media_categories))
            .route("/api/media/search", web::get().to(search_media))
            .route("/api/media/batch", web::put().to(batch_update_media))
            .route("/api/media/batch", web::delete().to(batch_delete_media))
            .route("/api/search", web::get().to(search))
            .route("/api/admin/users", web::get().to(get_admin_users))
            .route("/api/admin/users/{id}", web::put().to(update_admin_user))
            .route("/api/admin/users/{id}", web::delete().to(delete_admin_user))
            .route("/api/admin/comments", web::get().to(get_admin_comments))
            .route("/api/admin/comments/batch/approve", web::put().to(batch_approve_comments))
            .route("/api/admin/comments/batch", web::delete().to(batch_delete_comments))
            .route("/api/admin/stats/overview", web::get().to(get_stats_overview))
            .route("/api/admin/stats/visits", web::get().to(get_stats_visits))
            .route("/api/admin/stats/users", web::get().to(get_stats_users))
            // 安全相关路由
            .route("/api/admin/security/logs", web::get().to(get_security_logs))
            // SEO 相关路由
            .route("/sitemap.xml", web::get().to(generate_sitemap))
            .route("/robots.txt", web::get().to(generate_robots))
            .route("/api/posts/{id}/seo", web::put().to(update_post_seo_metadata))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
