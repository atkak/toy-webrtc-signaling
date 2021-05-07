use actix::{prelude::*, Actor, Addr, AsyncContext, StreamHandler, Supervised};
use actix_web_actors::ws;
use serde_json::Value;

use self::join::*;

mod join;

pub struct ClientSession {}

impl ClientSession {
    pub fn new() -> Self {
        ClientSession {}
    }
}

impl Actor for ClientSession {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ClientSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                let value: Value = serde_json::from_str(&text).unwrap();
                match &value["event"] {
                    Value::String(event) if event == "join" => handle_join(value, ctx.address()),
                    Value::String(event) if event == "offer" => {
                        let value: Value = serde_json::from_str(&text).unwrap();
                        println!("offer from: {}", value["from"].as_str().unwrap());
                        ctx.text(text);
                    }
                    Value::String(event) if event == "answer" => {
                        let value: Value = serde_json::from_str(&text).unwrap();
                        println!("answer from: {}", value["from"].as_str().unwrap());
                        ctx.text(text);
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }
}

#[derive(Default)]
struct Room {
    members: Vec<Addr<ClientSession>>,
}

impl Actor for Room {
    type Context = Context<Self>;
}

impl Supervised for Room {}
impl SystemService for Room {}
