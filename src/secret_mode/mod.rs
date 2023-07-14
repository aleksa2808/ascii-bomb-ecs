use bevy::{ecs as bevy_ecs, prelude::*};

use crate::{
    game::{
        add_common_game_systems,
        systems::{resize_window, setup_penguin_portraits, spawn_cameras},
        Set,
    },
    AppState,
};

use self::{resources::SecretModeMusic, systems::*};

mod resources;
mod systems;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
enum SecretModeLabel {
    Setup,
}

pub struct SecretModePlugin;

impl Plugin for SecretModePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SecretModeMusic>()
            .add_systems(
                OnEnter(AppState::SecretModeSetup),
                (setup_secret_mode, apply_deferred)
                    .chain()
                    .in_set(SecretModeLabel::Setup),
            )
            .add_systems(
                OnEnter(AppState::SecretModeSetup),
                (resize_window, apply_deferred)
                    .chain()
                    .after(SecretModeLabel::Setup),
            )
            .add_systems(
                OnEnter(AppState::SecretModeSetup),
                (spawn_cameras, apply_deferred)
                    .chain()
                    .after(SecretModeLabel::Setup),
            )
            .add_systems(
                Update,
                (secret_mode_manager, apply_deferred)
                    .chain()
                    .run_if(in_state(AppState::SecretModeManager)),
            )
            .add_systems(
                OnEnter(AppState::SecretModeTeardown),
                (teardown, apply_deferred).chain(),
            );

        app.add_systems(
            OnEnter(AppState::SecretModeInGame),
            (setup_penguin_portraits, apply_deferred).chain(),
        );
        add_common_game_systems(app, AppState::SecretModeInGame);
        app.add_systems(
            Update,
            (update_secret_mode, apply_deferred)
                .chain()
                .run_if(in_state(AppState::SecretModeInGame)),
        )
        .add_systems(
            Update,
            (finish_secret_mode, apply_deferred)
                .chain()
                .after(Set::PlayerMovement)
                .before(Set::FireSpawn)
                .run_if(in_state(AppState::SecretModeInGame)),
        );
    }
}
