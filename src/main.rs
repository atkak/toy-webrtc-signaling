use actix_files::Files;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

use crate::signaling::*;

mod signaling;
mod stun;

async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(ClientSession::new(), &req, stream);
    println!("{:?}", resp);
    resp
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/ws", web::get().to(index))
            .service(Files::new("/", "./web").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
