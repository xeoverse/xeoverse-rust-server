use std::collections::HashMap;
use std::str;

use actix::prelude::*;
use actix_web::web::Bytes;
use rand::{self, rngs::ThreadRng, Rng};
use serde_json::json;

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
    pub text: Option<String>,
    pub bytes: Option<Bytes>,
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

        let data: serde_json::Value = json!(state::get_all_users_states());
        let init_response = "userInit".to_owned() + " " + &id.to_string() + " " + &data.to_string();
        msg.addr.do_send(Message(init_response));

        self.sessions.insert(id, msg.addr);

        let join_response = "userJoin".to_owned() + " " + &id.to_string();
        self.emit_message(&join_response, 0);

        id
    }
}

impl Handler<Disconnect> for SocketManager {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");

        self.sessions.remove(&msg.id);

        state::remove_user_position(msg.id);

        let leave_response = "userLeave".to_owned() + " " + &msg.id.to_string();
        self.emit_message(&leave_response, msg.id);
    }
}

impl Handler<ClientMessage> for SocketManager {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        if msg.text.is_some() && msg.bytes.is_none() {
            let mut text = msg.text.as_ref().unwrap().split_whitespace();

            let msg_type = text.next().unwrap();
            let data = text.next().unwrap();

            match msg_type {
                "userMove" => {
                    let position: Vec<f32> = data.split(',').map(|s| s.parse().unwrap()).collect();
                    let floats = [position[0], position[1], position[2]];

                    state::update_user_position(msg.id, floats);

                    let response = "userMove".to_owned() + " " + &msg.id.to_string() + " " + &data;
                    self.emit_message(response.as_str(), msg.id);
                }
                "userRotate" => {
                    let rotation: Vec<f32> = data.split(',').map(|s| s.parse().unwrap()).collect();
                    let floats = [rotation[0], rotation[1], rotation[2]];

                    state::update_user_rotation(msg.id, floats);

                    let response =
                        "userRotate".to_owned() + " " + " " + &msg.id.to_string() + " " + &data;
                    self.emit_message(response.as_str(), msg.id);
                }
                _ => println!("Unknown action {}", msg_type),
            }
        } else if msg.bytes.is_some() && msg.text.is_none() {
            let bytes = msg.bytes.as_ref().unwrap();

            // let s = match str::from_utf8(bytes) {
            //     Ok(v) => v,
            //     Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            // };

            print!("Received bytes: {}", bytes.len())
        } else {
            println!("Invalid message");
        }
    }
}
