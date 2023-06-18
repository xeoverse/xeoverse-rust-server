use std::collections::HashMap;

use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use serde_json::{from_str, json, to_string};

use crate::state;

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

        msg.addr.do_send(Message(
            to_string(&json!({
                "type": "userInit",
                "userId": id,
                "userPositions": state::get_user_positions()
            }))
            .unwrap(),
        ));

        self.sessions.insert(id, msg.addr);

        self.emit_message(
            &to_string(&json!({
                "type": "userJoin",
                "userId": id
            }))
            .unwrap(),
            0,
        );

        id
    }
}

impl Handler<Disconnect> for SocketManager {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");

        self.sessions.remove(&msg.id);

        state::remove_user_position(msg.id);

        self.emit_message(
            &to_string(&json!({
                "type": "userLeave",
                "userId": msg.id
            }))
            .unwrap(),
            msg.id,
        );
    }
}

impl Handler<ClientMessage> for SocketManager {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        let json: Result<serde_json::Value, serde_json::Error> = from_str(&msg.msg);

        match json {
            Ok(json) => {
                let json: &serde_json::Map<String, serde_json::Value> = json.as_object().unwrap();

                let action: &str = json.get("type").unwrap().as_str().unwrap();

                match action {
                    "userMove" => {
                        let position: &Vec<serde_json::Value> =
                            json.get("position").unwrap().as_array().unwrap();

                        let response: serde_json::Value = json!({
                            "type": "userMove",
                            "position": position,
                            "userId": msg.id
                        });

                        let float_position: [f64; 3] = [
                            position[0].as_f64().unwrap(),
                            position[1].as_f64().unwrap(),
                            position[2].as_f64().unwrap(),
                        ];

                        state::update_user_position(msg.id, float_position);

                        self.emit_message(&to_string(&response).unwrap(), msg.id);
                    }
                    _ => println!("Unknown action {}", action),
                }
            }
            Err(e) => println!("Error parsing JSON: {}", e),
        }
    }
}
