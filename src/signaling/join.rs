use actix::{prelude::*, Addr};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::{ClientSession, Room};

pub fn handle_join(value: Value, addr: Addr<ClientSession>) {
    let body = value["body"].clone();
    let join: JoinEvent = serde_json::from_value(body).unwrap();
    println!("join from: {}", join.username);

    let msg = RoomJoinMessage::Join {
        username: join.username,
        addr,
    };

    let room_addr = Room::from_registry();
    room_addr.do_send(msg);
}

pub fn handle_leave(value: Value, addr: Addr<ClientSession>) {
    let username = value["from"].as_str().unwrap().to_string();
    println!("leave from: {}", username);

    let msg = RoomJoinMessage::Leave { username, addr };

    let room_addr = Room::from_registry();
    room_addr.do_send(msg);
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
enum RoomJoinMessage {
    Join {
        username: String,
        addr: Addr<ClientSession>,
    },
    Joined {
        username: String,
    },
    Leave {
        username: String,
        addr: Addr<ClientSession>,
    },
    Left {
        username: String,
    },
    Full,
    AlreadyJoined,
}

#[derive(Serialize, Deserialize)]
struct JoinEvent {
    username: String,
}

#[derive(Serialize, Deserialize)]
struct JoinedEvent {
    username: String,
}

impl Handler<RoomJoinMessage> for ClientSession {
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
            RoomJoinMessage::Left { username } => {
                let value = json!( {
                    "event": "left",
                    "from": username,
                });
                let json = serde_json::to_string(&value).unwrap();

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

                let remote_addr = self.find_opposite_addr(addr_ref).unwrap();
                remote_addr.do_send(RoomJoinMessage::Joined { username });
            }
            RoomJoinMessage::Leave { username, addr: _ } => {
                let msg = RoomJoinMessage::Left { username };
                self.members
                    .iter()
                    .for_each(move |a| a.do_send(msg.clone()));

                self.members.clear();
            }
            _ => (),
        }
    }
}
