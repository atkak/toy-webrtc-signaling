use actix::{prelude::*, Actor, Addr, AsyncContext, StreamHandler, Supervised};
use actix_web_actors::ws;
use serde_json::Value;

use self::exchange::*;
use self::join::*;

mod exchange;
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
                    Value::String(event) if event == "offer" => handle_offer(value, ctx),
                    Value::String(event) if event == "answer" => handle_answer(value, ctx),
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

impl Room {
    fn find_opposite_addr(&self, addr: &Addr<ClientSession>) -> Option<Addr<ClientSession>> {
        if self.members.len() != 2 {
            return None;
        }

        self.members.iter().find(|&e| e != addr).map(|v| v.clone())
    }
}

impl Actor for Room {
    type Context = Context<Self>;
}

impl Supervised for Room {}
impl SystemService for Room {}
