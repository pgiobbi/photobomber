use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, http, web};
use actix_web::middleware::NormalizePath;
use rand;
use rand::random_range;
use serde_json::json;

#[get("/count")]
async fn hello() -> impl Responder {
    let count = random_range(0..=20);
    HttpResponse::Ok().json(json!({
        "count": count
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin() // TODO: update
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                    .allowed_header(http::header::CONTENT_TYPE)
                    .max_age(3600),
            )
            .wrap(NormalizePath::trim())
            .service(web::scope("/api").service(hello))
    })
    .bind(("0.0.0.0", 3002))?
    .run()
    .await
}
