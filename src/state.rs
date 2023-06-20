use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Mutex};

static USER_ROTATIONS: Lazy<Mutex<HashMap<usize, [f32; 3]>>> = Lazy::new(|| {
    let m: HashMap<usize, [f32; 3]> = HashMap::new();
    Mutex::new(m)
});

pub fn update_user_rotation(user_id: usize, rotation: [f32; 3]) {
    let old_rotation = USER_ROTATIONS
        .lock()
        .unwrap()
        .get(&user_id)
        .copied()
        .unwrap_or([0.0; 3]);
    let new_rotation = [
        old_rotation[0] + rotation[0],
        old_rotation[1] + rotation[1],
        old_rotation[2] + rotation[2],
    ];
    USER_ROTATIONS.lock().unwrap().insert(user_id, new_rotation);
}

static USER_POSITIONS: Lazy<Mutex<HashMap<usize, [f32; 3]>>> = Lazy::new(|| {
    let m: HashMap<usize, [f32; 3]> = HashMap::new();
    Mutex::new(m)
});

pub fn update_user_position(user_id: usize, position: [f32; 3]) {
    let old_position = USER_POSITIONS.lock().unwrap().get(&user_id).copied();

    let new_position = match old_position {
        Some(old_position) => {
            let new_position = [
                old_position[0] + position[0],
                old_position[1] + position[1],
                old_position[2] + position[2],
            ];
            new_position
        }
        None => position,
    };

    USER_POSITIONS.lock().unwrap().insert(user_id, new_position);
}

pub fn remove_user_position(user_id: usize) {
    USER_POSITIONS.lock().unwrap().remove(&user_id);
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserState {
    pub user_id: usize,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
}

pub fn get_all_users_states() -> HashMap<usize, UserState> {
    let mut user_states: HashMap<usize, UserState>;

    let user_positions = USER_POSITIONS.lock().unwrap();
    let user_rotations = USER_ROTATIONS.lock().unwrap();

    user_states = HashMap::new();

    for (user_id, position) in user_positions.iter() {
        let rotation = user_rotations.get(user_id).copied().unwrap_or([0.0; 3]);
        user_states.insert(
            *user_id,
            UserState {
                user_id: *user_id,
                position: *position,
                rotation,
            },
        );
    }

    user_states
}
