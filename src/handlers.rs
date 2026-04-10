use actix_web::{web, HttpResponse, Responder};
use diesel::prelude::*;
use crate::{models::{Post, NewPost, UpdatePost}, schema::posts, db::establish_connection};

pub async fn get_posts() -> impl Responder {
    let conn = establish_connection();
    let results = posts::table.load::<Post>(&conn).expect("Error loading posts");
    HttpResponse::Ok().json(results)
}

pub async fn get_post(path: web::Path<i32>) -> impl Responder {
    let id = path.into_inner();
    let conn = establish_connection();
    match posts::table.find(id).first::<Post>(&conn) {
        Ok(post) => HttpResponse::Ok().json(post),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub async fn create_post(new_post: web::Json<NewPost>) -> impl Responder {
    let conn = establish_connection();
    let result = diesel::insert_into(posts::table)
        .values(&new_post)
        .execute(&conn)
        .expect("Error creating post");
    HttpResponse::Created().json(result)
}

pub async fn update_post(path: web::Path<i32>, updated_post: web::Json<UpdatePost>) -> impl Responder {
    let id = path.into_inner();
    let conn = establish_connection();
    match diesel::update(posts::table.find(id))
        .set(&updated_post)
        .execute(&conn)
    {
        Ok(affected) if affected > 0 => HttpResponse::Ok().json(affected),
        _ => HttpResponse::NotFound().finish(),
    }
}

pub async fn delete_post(path: web::Path<i32>) -> impl Responder {
    let id = path.into_inner();
    let conn = establish_connection();
    match diesel::delete(posts::table.find(id))
        .execute(&conn)
    {
        Ok(affected) if affected > 0 => HttpResponse::Ok().json(affected),
        _ => HttpResponse::NotFound().finish(),
    }
}
