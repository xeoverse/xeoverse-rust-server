use actix::prelude::*;
use actix_cors::Cors;
use actix_web::{
    get, http::header, middleware::Logger, web, App, Error, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use actix_web_actors::ws;
use std::env;
use std::time::Instant;

mod physics;
mod render_loop;
mod server;
mod session;
mod state;

async fn websocket_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::SocketManager>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        session::SocketSession {
            id: 0,
            hb: Instant::now(),
            name: None,
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}

#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a number");

    // render_loop::run();

    let host: String = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let origin: String = env::var("ORIGIN").unwrap_or_else(|_| "http://localhost:3000".to_string());

    let server: Addr<server::SocketManager> = server::SocketManager::new().start();

    HttpServer::new(move || {
        let cors: Cors = Cors::default()
            .allowed_origin(&origin)
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(&[header::AUTHORIZATION, header::ACCEPT])
            .allowed_header(header::CONTENT_TYPE)
            .expose_headers(&[header::CONTENT_DISPOSITION])
            .block_on_origin_mismatch(false)
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(server.clone()))
            .route("/ws", web::get().to(websocket_route))
            .service(hello)
            .wrap(Logger::default())
            .wrap(cors)
    })
    .workers(2)
    .bind((host, port))?
    .run()
    .await
}
