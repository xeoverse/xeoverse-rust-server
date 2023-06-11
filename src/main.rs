use actix_cors::Cors;
use actix_web::{get, http::header, App, HttpResponse, HttpServer, Responder};

#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors: Cors = Cors::default()
            .allowed_origin("http://127.0.0.1:8080")
            .allowed_origin_fn(|origin, _req_head| {
                origin.as_bytes().starts_with(b"http://localhost")
            })
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(&[header::AUTHORIZATION, header::ACCEPT])
            .allowed_header(header::CONTENT_TYPE)
            .expose_headers(&[header::CONTENT_DISPOSITION])
            .block_on_origin_mismatch(false)
            .max_age(3600);

        App::new().wrap(cors).service(hello)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
