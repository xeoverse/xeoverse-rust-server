use std::collections::HashMap;

use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use serde_json::{from_str, json, to_string};

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub id: usize,
    pub msg: String,
}

#[derive(Debug)]
pub struct SocketManager {
    sessions: HashMap<usize, Recipient<Message>>,
    rng: ThreadRng,
}

impl SocketManager {
    pub fn new() -> SocketManager {
        SocketManager {
            sessions: HashMap::new(),
            rng: rand::thread_rng(),
        }
    }

    fn emit_message(&self, message: &str, skip_id: usize) {
        for (id, addr) in self.sessions.iter() {
            if *id != skip_id {
                addr.do_send(Message(message.to_owned()));
            }
        }
    }
}

impl Actor for SocketManager {
    type Context = Context<Self>;
}

impl Handler<Connect> for SocketManager {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");

        let id: usize = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);

        id
    }
}

impl Handler<Disconnect> for SocketManager {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");

        self.sessions.remove(&msg.id);

        self.emit_message("Someone disconnected", 0);
    }
}

impl Handler<ClientMessage> for SocketManager {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.emit_message(msg.msg.as_str(), msg.id);

        println!("Got message from {}: {}", msg.id, msg.msg);

        let json: Result<serde_json::Value, serde_json::Error> = from_str(&msg.msg);

        match json {
            Ok(json) => {
                let json: &serde_json::Map<String, serde_json::Value> = json.as_object().unwrap();

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

                        self.emit_message(&to_string(&response).unwrap(), 0);
                    }
                    _ => println!("Unknown action {}", action),
                }
            }
            Err(e) => println!("Error parsing JSON: {}", e),
        }
    }
}
