use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use crate::handlers::{get_posts, get_post, create_post, update_post, delete_post, register, login, get_comments, create_comment, update_comment, delete_comment, get_categories, create_category, update_category, delete_category, get_tags, create_tag, update_tag, delete_tag, add_post_tags};

mod models;
mod schema;
mod db;
mod handlers;
mod auth;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        
        App::new()
            .wrap(cors)
            .route("/api/posts", web::get().to(get_posts))
            .route("/api/posts/{id}", web::get().to(get_post))
            .route("/api/posts", web::post().to(create_post))
            .route("/api/posts/{id}", web::put().to(update_post))
            .route("/api/posts/{id}", web::delete().to(delete_post))
            .route("/api/auth/register", web::post().to(register))
            .route("/api/auth/login", web::post().to(login))
            .route("/api/comments/{post_id}", web::get().to(get_comments))
            .route("/api/comments", web::post().to(create_comment))
            .route("/api/comments/{id}", web::put().to(update_comment))
            .route("/api/comments/{id}", web::delete().to(delete_comment))
            .route("/api/categories", web::get().to(get_categories))
            .route("/api/categories", web::post().to(create_category))
            .route("/api/categories/{id}", web::put().to(update_category))
            .route("/api/categories/{id}", web::delete().to(delete_category))
            .route("/api/tags", web::get().to(get_tags))
            .route("/api/tags", web::post().to(create_tag))
            .route("/api/tags/{id}", web::put().to(update_tag))
            .route("/api/tags/{id}", web::delete().to(delete_tag))
            .route("/api/posts/tags", web::post().to(add_post_tags))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
