use bevy::prelude::*;
use bevy_kira_audio::Audio;

use crate::common::types::InputAction;

use super::resources::{GameOption, GameOptionStore, InputActionStatusTracker};

pub fn set_volume_based_on_options(game_option_store: Res<GameOptionStore>, audio: Res<Audio>) {
    if game_option_store.get(GameOption::Sound) {
        audio.set_volume(1.0);
    } else {
        audio.set_volume(0.0);
    }
}

pub fn clear_inputs(mut inputs: ResMut<InputActionStatusTracker>) {
    inputs.clear();
}

pub fn handle_keyboard_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut inputs: ResMut<InputActionStatusTracker>,
) {
    static BINDINGS: &[(KeyCode, InputAction)] = &[
        (KeyCode::Up, InputAction::Up),
        (KeyCode::Down, InputAction::Down),
        (KeyCode::Left, InputAction::Left),
        (KeyCode::Right, InputAction::Right),
        (KeyCode::Space, InputAction::Space),
        (KeyCode::W, InputAction::W),
        (KeyCode::S, InputAction::S),
        (KeyCode::A, InputAction::A),
        (KeyCode::D, InputAction::D),
        (KeyCode::G, InputAction::G),
        (KeyCode::Return, InputAction::Return),
        (KeyCode::Escape, InputAction::Escape),
        (KeyCode::Back, InputAction::Back),
        (KeyCode::F, InputAction::F),
    ];

    for (key_code, input_action) in BINDINGS {
        if keyboard_input.just_pressed(*key_code) {
            inputs.set_active(*input_action);
        }
    }
}
