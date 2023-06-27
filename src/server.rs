use actix::prelude::*;
use actix_web::web::Bytes;
use serde_json::json;
use std::collections::HashMap;
use std::str;
use std::thread;
use std::time::Duration;

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
    current_user_id: usize,
}

// const MESSAGE_TYPES: [&str; 5] = [
//     "userInit",
//     "userJoin",
//     "userLeave",
//     "userMove",
//     "userRotate",
//     "userShoot"
// ];

impl SocketManager {
    pub fn new() -> SocketManager {
        SocketManager {
            sessions: HashMap::new(),
            current_user_id: 0,
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

        let id: usize = self.current_user_id + 1;
        self.current_user_id = id;

        let data: serde_json::Value = json!(state::get_all_users_states());

        let init_response = format!("{} {} {}", 0, id.to_string(), data);
        msg.addr.do_send(Message(init_response));

        self.sessions.insert(id, msg.addr);

        let join_response = format!("{} {}", 1, id.to_string());

        thread::sleep(Duration::from_secs(2));

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

        let leave_response = format!("{} {}", 2, msg.id.to_string());
        self.emit_message(&leave_response, msg.id);
    }
}

impl Handler<ClientMessage> for SocketManager {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        if msg.text.is_some() && msg.bytes.is_none() {
            let mut text = msg.text.as_ref().unwrap().split_whitespace();

            let msg_type = text.next().unwrap();
            let data1 = text.next().unwrap();

            match msg_type {
                "3" => {
                    let position: Vec<f32> = data1.split(',').map(|s| s.parse().unwrap()).collect();

                    state::update_user_position(msg.id, [position[0], position[1], position[2]]);

                    let move_response = format!("{} {} {}", 3, msg.id.to_string(), data1);

                    self.emit_message(&move_response, msg.id);
                }
                "4" => {
                    let rotation: Vec<f32> = data1.split(',').map(|s| s.parse().unwrap()).collect();

                    state::update_user_rotation(msg.id, [rotation[0], rotation[1], rotation[2]]);

                    let rotate_response = format!("{} {} {}", 4, msg.id.to_string(), data1);

                    self.emit_message(&rotate_response, msg.id);
                }
                "5" => {
                    let data2 = text.next().unwrap();
                    // let position: Vec<f32> = data1.split(',').map(|s| s.parse().unwrap()).collect();
                    // let direction: Vec<f32> = data2.split(',').map(|s| s.parse().unwrap()).collect();

                    let shoot_response = format!("{} {} {} {}", 5, msg.id.to_string(), data1, data2);

                    self.emit_message(&shoot_response, msg.id);
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
