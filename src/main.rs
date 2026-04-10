use actix_web::{web, App, HttpServer};
use crate::handlers::{get_posts, get_post, create_post, update_post, delete_post};

mod models;
mod schema;
mod db;
mod handlers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/api/posts", web::get().to(get_posts))
            .route("/api/posts/{id}", web::get().to(get_post))
            .route("/api/posts", web::post().to(create_post))
            .route("/api/posts/{id}", web::put().to(update_post))
            .route("/api/posts/{id}", web::delete().to(delete_post))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
