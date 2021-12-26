use bevy::{core::Timer, prelude::*};
use bevy_kira_audio::AudioSource;

use crate::{game::types::Cooldown, loading::resources::AssetsLoading};

pub struct SecretModeMusic {
    pub what_is_f: Handle<AudioSource>,
}

impl FromWorld for SecretModeMusic {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        let secret_mode_music = SecretModeMusic {
            what_is_f: asset_server.load("sounds/what_is_f.ogg"),
        };

        if let Some(mut assets_loading) = world.get_resource_mut::<AssetsLoading>() {
            assets_loading
                .0
                .push(secret_mode_music.what_is_f.clone_untyped());
        }

        secret_mode_music
    }
}

pub enum SecretModeManagerState {
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
    pub manager_state: SecretModeManagerState,
    pub in_game_state: SecretModeInGameState,
    pub pattern: &'static str,
}
