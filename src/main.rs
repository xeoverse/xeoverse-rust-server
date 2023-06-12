use actix::{Actor, StreamHandler};
use actix_cors::Cors;
use actix_web::{
    get, http::header, middleware::Logger, web, App, Error, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use actix_web_actors::ws;
use serde_json::{from_str, json, to_string};
use std::env;

/// Define HTTP actor
struct WebSocket;

impl Actor for WebSocket {
    type Context = ws::WebsocketContext<Self>;
}

fn parse_json(text: &str) -> serde_json::Result<serde_json::Value> {
    let v: serde_json::Value = from_str(text)?;
    println!("Parsed JSON: {:?}", v);
    Ok(v)
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        println!("{:?}", msg);
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                let json: Result<serde_json::Value, serde_json::Error> = parse_json(&text);
                match json {
                    Ok(json) => {
                        let json: &serde_json::Map<String, serde_json::Value> =
                            json.as_object().unwrap();

                        let action: &str = json.get("type").unwrap().as_str().unwrap();

                        match action {
                            "move" => {
                                let direction: &Vec<serde_json::Value> =
                                    json.get("direction").unwrap().as_array().unwrap();
                                let user_id: &str = json.get("userId").unwrap().as_str().unwrap();

                                let response: serde_json::Value = json!({
                                    "type": "move",
                                    "direction": direction,
                                    "userId": user_id
                                });

                                ctx.text(to_string(&response).unwrap());
                            }
                            _ => println!("Unknown action {}", action),
                        }
                    }
                    Err(e) => println!("Error parsing JSON: {}", e),
                }
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

async fn websocket_route(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp: Result<HttpResponse, Error> = ws::start(WebSocket {}, &req, stream);
    println!("{:?}", resp);
    resp
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

    let host: String = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let origin: String = env::var("ORIGIN").unwrap_or_else(|_| "http://localhost:3000".to_string());

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
            .wrap(cors)
            .route("/ws", web::get().to(websocket_route))
            .service(hello)
            .wrap(Logger::default())
    })
    .bind((host, port))?
    .run()
    .await
}
