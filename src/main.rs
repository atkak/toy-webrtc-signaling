use actix::{prelude::*, Actor, Addr, AsyncContext, StreamHandler, Supervised};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

mod stun;

#[derive(Message)]
#[rtype(result = "()")]
enum RoomJoinMessage {
    Join {
        username: String,
        addr: Addr<WsServer>,
    },
    Joined {
        username: String,
    },
    Full,
    AlreadyJoined,
}

#[derive(Default)]
struct Room {
    members: Vec<Addr<WsServer>>,
}

impl Actor for Room {
    type Context = Context<Self>;
}

impl Supervised for Room {}
impl SystemService for Room {}

impl Handler<RoomJoinMessage> for Room {
    type Result = ();

    fn handle(&mut self, msg: RoomJoinMessage, _: &mut Self::Context) {
        match msg {
            RoomJoinMessage::Join { username, addr } => {
                let addr_ref = &addr;

                if !self.members.contains(addr_ref) && self.members.len() > 2 {
                    addr.do_send(RoomJoinMessage::Full);
                    return;
                }

                if self.members.contains(addr_ref) {
                    addr.do_send(RoomJoinMessage::AlreadyJoined);
                    return;
                }

                self.members.push(addr.clone());

                if self.members.len() <= 1 {
                    return;
                }

                let remote_addr = self.members.iter().find(|&e| e != addr_ref).unwrap();

                remote_addr.do_send(RoomJoinMessage::Joined { username });
            }
            _ => (),
        }
    }
}

struct WsServer {}

impl WsServer {
    fn new() -> Self {
        Self {}
    }
}

impl Actor for WsServer {
    type Context = ws::WebsocketContext<Self>;
}

impl Handler<RoomJoinMessage> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: RoomJoinMessage, ctx: &mut Self::Context) {
        match msg {
            RoomJoinMessage::Joined { username } => {
                let joined = JoinedEvent {
                    username: username.clone(),
                };
                let joined_value = json!( {
                    "event": "joined",
                    "from": username,
                    "body": joined,
                });
                let json = serde_json::to_string(&joined_value).unwrap();

                ctx.text(json);
            }
            RoomJoinMessage::Full => {
                let joined_value = json!( {
                    "event": "room_full",
                });
                let json = serde_json::to_string(&joined_value).unwrap();

                ctx.text(json);
            }
            RoomJoinMessage::AlreadyJoined => {
                let joined_value = json!( {
                    "event": "room_full",
                });
                let json = serde_json::to_string(&joined_value).unwrap();

                ctx.text(json);
            }
            _ => (),
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsServer {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                let value: Value = serde_json::from_str(&text).unwrap();
                match &value["event"] {
                    Value::String(event) if event == "join" => {
                        let body = value["body"].clone();
                        let join: JoinEvent = serde_json::from_value(body).unwrap();
                        println!("join from: {}", join.username);

                        let msg = RoomJoinMessage::Join {
                            username: join.username,
                            addr: ctx.address(),
                        };

                        let room_addr = Room::from_registry();
                        room_addr.do_send(msg);
                    }
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

#[derive(Serialize, Deserialize)]
struct JoinEvent {
    username: String,
}

#[derive(Serialize, Deserialize)]
struct JoinedEvent {
    username: String,
}

async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(WsServer::new(), &req, stream);
    println!("{:?}", resp);
    resp
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/ws", web::get().to(index)))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
