use std::{sync::Mutex, collections::HashMap};
use once_cell::sync::Lazy;

static USER_POSITIONS: Lazy<Mutex<HashMap<usize, [f64; 3]>>> = Lazy::new(|| {
    let m: HashMap<usize, [f64; 3]> = HashMap::new();
    Mutex::new(m)
});

pub fn get_user_positions() -> HashMap<usize, [f64; 3]> {
    USER_POSITIONS.lock().unwrap().clone()
}

pub fn get_user_position(user_id: usize) -> Option<[f64; 3]> {
    USER_POSITIONS.lock().unwrap().get(&user_id).copied()
}

pub fn set_user_position(user_id: usize, position: [f64; 3]) {
    USER_POSITIONS.lock().unwrap().insert(user_id, position);
}

pub fn update_user_position(user_id: usize, position: [f64; 3]) {
    let old_position = USER_POSITIONS.lock().unwrap().get(&user_id).copied();

    let new_position = match old_position {
        Some(old_position) => {
            let new_position = [
                old_position[0] + position[0],
                old_position[1] + position[1],
                old_position[2] + position[2],
            ];
            println!("{:?}", new_position);
            new_position
        },
        None => {
            println!("{:?}", position);
            position
        },
    };

    USER_POSITIONS.lock().unwrap().insert(user_id, new_position);
}

pub fn remove_user_position(user_id: usize) {
    USER_POSITIONS.lock().unwrap().remove(&user_id);
}