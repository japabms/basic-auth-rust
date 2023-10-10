pub mod auth;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use auth::BasicAuth;
use std::fs;

#[get("/")]
async fn index() -> impl Responder {
    let index = fs::read("./index.html").unwrap();
    HttpResponse::Ok().body(index)
}

#[get("/test")]
async fn test() -> impl Responder {
    let page = fs::read("./curriculo.html").unwrap();
    HttpResponse::Ok().body(page)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(test)
            .service(web::scope("").wrap(BasicAuth).service(index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
