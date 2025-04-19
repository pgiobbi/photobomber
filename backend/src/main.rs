use actix_cors::Cors;
use actix_web::middleware::NormalizePath;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, http, web};
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
                    .allowed_origin_fn(|origin, _req_head| {
                        // Extract the origin string
                        let origin_str = origin.to_str().unwrap();

                        // Allow localhost variations (http://localhost, http://127.0.0.1, http://[::1], any port)
                        origin_str.starts_with("http://localhost") ||
                        origin_str.starts_with("http://127.0.0.1") ||
                        origin_str.starts_with("http://[::1]") ||
                        // Allow https://photobomber.servebeer.com
                        origin_str == "https://photobomber.servebeer.com"
                    })
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
