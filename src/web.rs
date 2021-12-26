use bevy::prelude::*;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::AppState;

pub static START: Lazy<RwLock<bool>> = Lazy::new(|| RwLock::new(false));

#[wasm_bindgen]
#[allow(dead_code)]
pub fn start_game() {
    let mut start = START.write();
    *start = true;
}

#[wasm_bindgen(module = "/web/index.js")]
extern "C" {
    pub fn doneLoading();
}

// web-specific systems
pub fn web_ready_to_start_enter() {
    // TODO: would it be better to do this through web-sys?
    doneLoading();
}

pub fn web_ready_to_start_update(mut state: ResMut<State<AppState>>) {
    if *START.read() {
        state.replace(AppState::SplashScreen).unwrap();
    }
}
