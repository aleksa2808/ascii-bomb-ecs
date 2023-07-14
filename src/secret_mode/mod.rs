use bevy::prelude::*;

use crate::{
    game::{
        common_game_systems,
        systems::{resize_window, setup_penguin_portraits, spawn_cameras},
        Set,
    },
    AppState,
};

use self::{resources::SecretModeMusic, systems::*};

mod resources;
mod systems;

pub struct SecretModePlugin;

impl Plugin for SecretModePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SecretModeMusic>()
            .add_systems(
                OnEnter(AppState::SecretModeSetup),
                (
                    setup_secret_mode,
                    apply_deferred,
                    (resize_window, spawn_cameras),
                )
                    .chain(),
            )
            .add_systems(
                Update,
                secret_mode_manager.run_if(in_state(AppState::SecretModeManager)),
            )
            .add_systems(OnEnter(AppState::SecretModeTeardown), teardown);

        app.add_systems(OnEnter(AppState::SecretModeInGame), setup_penguin_portraits);
        app.add_systems(
            Update,
            (
                common_game_systems(),
                (
                    update_secret_mode,
                    apply_deferred,
                    finish_secret_mode
                        .after(Set::PlayerMovement)
                        .before(Set::FireSpawn),
                )
                    .chain(),
            )
                .run_if(in_state(AppState::SecretModeInGame)),
        );
    }
}
