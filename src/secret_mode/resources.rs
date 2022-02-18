use bevy::{core::Timer, prelude::*};

use crate::{
    audio::{SoundHandles, SoundID},
    game::types::Cooldown,
    loading::resources::AssetsLoading,
};

pub struct SecretModeMusic {
    pub what_is_f: SoundID,
}

impl FromWorld for SecretModeMusic {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        let what_is_f_handle = asset_server.load("sounds/what_is_f.ogg");

        if let Some(mut assets_loading) = world.get_resource_mut::<AssetsLoading>() {
            assets_loading.0.push(what_is_f_handle.clone_untyped());
        }

        let mut sound_handles = world.get_resource_mut::<SoundHandles>().unwrap();
        SecretModeMusic {
            what_is_f: sound_handles.add_handle(what_is_f_handle),
        }
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
