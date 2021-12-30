use std::collections::{HashMap, VecDeque};

use bevy::prelude::*;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    common::{resources::InputActionStatusTracker, types::InputAction},
    AppState,
};

static START: Lazy<RwLock<bool>> = Lazy::new(|| RwLock::new(false));
static INPUTS: Lazy<RwLock<VecDeque<u8>>> = Lazy::new(|| RwLock::new(VecDeque::new()));

// functions callable from JavaScript
#[wasm_bindgen]
#[allow(dead_code)]
pub fn start_game() {
    let mut start = START.write();
    *start = true;
}

#[wasm_bindgen]
#[allow(dead_code)]
pub fn set_input_active(input: u8) {
    let mut inputs = INPUTS.write();
    inputs.push_front(input);
}

// callable JavaScript functions
#[wasm_bindgen(module = "/web/wasm_callables.js")]
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

pub fn handle_web_input(mut input_actions: ResMut<InputActionStatusTracker>) {
    let bindings: HashMap<u8, InputAction> = vec![
        (0, InputAction::Up),
        (1, InputAction::Down),
        (2, InputAction::Left),
        (3, InputAction::Right),
        (4, InputAction::Space),
        (5, InputAction::W),
        (6, InputAction::S),
        (7, InputAction::A),
        (8, InputAction::D),
        (9, InputAction::G),
        (10, InputAction::Return),
        (11, InputAction::Escape),
        (12, InputAction::Back),
        (13, InputAction::F),
    ]
    .into_iter()
    .collect();

    let mut inputs = INPUTS.write();
    while let Some(input) = inputs.pop_back() {
        if let Some(input_action) = bindings.get(&input) {
            input_actions.set_active(*input_action);
        }
    }
}
