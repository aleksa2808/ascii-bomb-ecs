use bevy::prelude::*;
use bevy_kira_audio::Audio;

use super::resources::{GameOption, GameOptionStore};

pub fn set_volume_based_on_options(game_option_store: Res<GameOptionStore>, audio: Res<Audio>) {
    if game_option_store.get(GameOption::Sound) {
        audio.set_volume(1.0);
    } else {
        audio.set_volume(0.0);
    }
}
