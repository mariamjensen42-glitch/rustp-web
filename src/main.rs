use actix_web::{web, App, HttpServer, HttpRequest, HttpMessage};
use actix_cors::Cors;
use std::sync::Mutex;
use std::collections::HashSet;
use crate::handlers::{
    get_posts, get_post, get_post_by_slug, create_post, update_post, delete_post, update_post_status,
    register, login, logout, refresh_token, get_me, update_me, update_password,
    get_comments, create_comment, approve_comment, delete_comment,
    get_categories, get_category, create_category, update_category, delete_category, get_category_posts,
    get_tags, create_tag, update_tag, delete_tag, get_tag_posts, add_post_tags,
    upload_media, get_media, delete_media,
    search, add_post_tags as add_post_tags_handler, AppState,
};

mod models;
mod schema;
mod db;
mod handlers;
mod auth;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        blacklist: Mutex::new(HashSet::new()),
    });

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
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
