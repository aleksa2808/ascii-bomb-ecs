use bevy::prelude::*;

use crate::{audio::Audio, common::types::InputAction};

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
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut inputs: ResMut<InputActionStatusTracker>,
) {
    static BINDINGS: &[(KeyCode, InputAction)] = &[
        (KeyCode::ArrowUp, InputAction::Up),
        (KeyCode::ArrowDown, InputAction::Down),
        (KeyCode::ArrowLeft, InputAction::Left),
        (KeyCode::ArrowRight, InputAction::Right),
        (KeyCode::Space, InputAction::Space),
        (KeyCode::KeyW, InputAction::W),
        (KeyCode::KeyS, InputAction::S),
        (KeyCode::KeyA, InputAction::A),
        (KeyCode::KeyD, InputAction::D),
        (KeyCode::KeyG, InputAction::G),
        (KeyCode::Enter, InputAction::Return),
        (KeyCode::Escape, InputAction::Escape),
        (KeyCode::Backspace, InputAction::Back),
        (KeyCode::KeyF, InputAction::F),
    ];

    for (key_code, input_action) in BINDINGS {
        if keyboard_input.just_pressed(*key_code) {
            inputs.set_active(*input_action);
        }
    }
}
