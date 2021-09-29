use bevy::{core::Timer, prelude::*};
use bevy_kira_audio::AudioSource;

use crate::types::Cooldown;

pub struct SecretModeMusic {
    pub what_is_f: Handle<AudioSource>,
}

impl FromWorld for SecretModeMusic {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        SecretModeMusic {
            what_is_f: asset_server.load("sounds/what_is_f.ogg"),
        }
    }
}

pub enum SecretModeDispatcherState {
    Setup,
    MapTransition,
    InGame,
}

pub enum SecretModeInGameState {
    Initial(Timer),
    Started {
        move_cooldown: Cooldown,
        round_progress: usize,
        round: usize,
    },
    Stopping(Timer),
}

pub struct SecretModeContext {
    pub dispatcher_state: SecretModeDispatcherState,
    pub in_game_state: SecretModeInGameState,
    pub pattern: &'static str,
}
