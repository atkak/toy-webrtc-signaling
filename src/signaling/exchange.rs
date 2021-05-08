use actix::{prelude::*, Addr, AsyncContext};
use actix_web_actors::ws;
use serde_json::{json, Value};

use super::{ClientSession, Room};

pub fn handle_offer(value: Value, ctx: &mut ws::WebsocketContext<ClientSession>) {
    println!("offer from: {}", value["from"].as_str().unwrap());

    let username = value["from"].as_str().unwrap().to_string();
    let sdp = value["body"].clone();
    let msg = SdpExchangeMessage::Offer {
        username,
        sdp,
        addr: ctx.address(),
    };

    let room_addr = Room::from_registry();
    room_addr.do_send(msg)
}

pub fn handle_answer(value: Value, ctx: &mut ws::WebsocketContext<ClientSession>) {
    println!("answer from: {}", value["from"].as_str().unwrap());

    let username = value["from"].as_str().unwrap().to_string();
    let sdp = value["body"].clone();
    let msg = SdpExchangeMessage::Answer {
        username,
        sdp,
        addr: ctx.address(),
    };

    let room_addr = Room::from_registry();
    room_addr.do_send(msg)
}

pub fn handle_icecandidate(value: Value, ctx: &mut ws::WebsocketContext<ClientSession>) {
    println!("icecandidate from: {}", value["from"].as_str().unwrap());

    let username = value["from"].as_str().unwrap().to_string();
    let candidate = value["body"].clone();
    let msg = IceCandidateExchangeMessage::IceCandidate {
        username,
        candidate,
        addr: ctx.address(),
    };

    let room_addr = Room::from_registry();
    room_addr.do_send(msg)
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
enum SdpExchangeMessage {
    Offer {
        username: String,
        sdp: Value,
        addr: Addr<ClientSession>,
    },
    Answer {
        username: String,
        sdp: Value,
        addr: Addr<ClientSession>,
    },
    NoMembers,
}

impl Handler<SdpExchangeMessage> for ClientSession {
    type Result = ();

    fn handle(&mut self, msg: SdpExchangeMessage, ctx: &mut Self::Context) {
        match msg {
            SdpExchangeMessage::Offer {
                username,
                sdp,
                addr: _,
            } => {
                let value = json!( {
                    "event": "offer",
                    "from": username,
                    "body": sdp,
                });
                let json = serde_json::to_string(&value).unwrap();

                ctx.text(json);
            }
            SdpExchangeMessage::Answer {
                username,
                sdp,
                addr: _,
            } => {
                let value = json!( {
                    "event": "answer",
                    "from": username,
                    "body": sdp,
                });
                let json = serde_json::to_string(&value).unwrap();

                ctx.text(json);
            }
            SdpExchangeMessage::NoMembers => {
                let value = json!( {
                    "event": "nomembers",
                });
                let json = serde_json::to_string(&value).unwrap();

                ctx.text(json);
            }
        }
    }
}

impl Handler<SdpExchangeMessage> for Room {
    type Result = ();

    fn handle(&mut self, msg: SdpExchangeMessage, _ctx: &mut Self::Context) {
        let forward_msg = msg.clone();
        match msg {
            SdpExchangeMessage::Offer {
                username: _,
                sdp: _,
                addr,
            } => {
                if self.members.len() < 2 {
                    addr.do_send(SdpExchangeMessage::NoMembers);
                    return;
                }

                let remote_addr = self.find_opposite_addr(&addr).unwrap();
                remote_addr.do_send(forward_msg);
            }
            SdpExchangeMessage::Answer {
                username: _,
                sdp: _,
                addr,
            } => {
                let remote_addr = self.find_opposite_addr(&addr).unwrap();
                remote_addr.do_send(forward_msg);
            }
            _ => (),
        }
    }
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
enum IceCandidateExchangeMessage {
    IceCandidate {
        username: String,
        candidate: Value,
        addr: Addr<ClientSession>,
    },
    NoMembers,
}

impl Handler<IceCandidateExchangeMessage> for ClientSession {
    type Result = ();

    fn handle(&mut self, msg: IceCandidateExchangeMessage, ctx: &mut Self::Context) {
        match msg {
            IceCandidateExchangeMessage::IceCandidate {
                username,
                candidate,
                addr: _,
            } => {
                let value = json!( {
                    "event": "icecandidate",
                    "from": username,
                    "body": candidate,
                });
                let json = serde_json::to_string(&value).unwrap();

                ctx.text(json);
            }
            IceCandidateExchangeMessage::NoMembers => {
                let value = json!( {
                    "event": "nomembers",
                });
                let json = serde_json::to_string(&value).unwrap();

                ctx.text(json);
            }
        }
    }
}

impl Handler<IceCandidateExchangeMessage> for Room {
    type Result = ();

    fn handle(&mut self, msg: IceCandidateExchangeMessage, _ctx: &mut Self::Context) {
        let forward_msg = msg.clone();
        match msg {
            IceCandidateExchangeMessage::IceCandidate {
                username: _,
                candidate: _,
                addr,
            } => {
                if self.members.len() < 2 {
                    addr.do_send(SdpExchangeMessage::NoMembers);
                    return;
                }

                let remote_addr = self.find_opposite_addr(&addr).unwrap();
                remote_addr.do_send(forward_msg);
            }
            _ => (),
        }
    }
}
