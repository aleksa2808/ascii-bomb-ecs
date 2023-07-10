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
                (setup_secret_mode, apply_system_buffers)
                    .chain()
                    .in_set(SecretModeLabel::Setup)
                    .in_schedule(OnEnter(AppState::SecretModeSetup)),
            )
            .add_systems(
                (resize_window, apply_system_buffers)
                    .chain()
                    .after(SecretModeLabel::Setup)
                    .in_schedule(OnEnter(AppState::SecretModeSetup)),
            )
            .add_systems(
                (spawn_cameras, apply_system_buffers)
                    .chain()
                    .after(SecretModeLabel::Setup)
                    .in_schedule(OnEnter(AppState::SecretModeSetup)),
            )
            .add_systems(
                (secret_mode_manager, apply_system_buffers)
                    .chain()
                    .in_set(OnUpdate(AppState::SecretModeManager)),
            )
            .add_systems(
                (teardown, apply_system_buffers)
                    .chain()
                    .in_schedule(OnEnter(AppState::SecretModeTeardown)),
            );

        app.add_systems(
            (setup_penguin_portraits, apply_system_buffers)
                .chain()
                .in_schedule(OnEnter(AppState::SecretModeInGame)),
        );
        add_common_game_systems(app, AppState::SecretModeInGame);
        app.add_systems(
            (update_secret_mode, apply_system_buffers)
                .chain()
                .in_set(OnUpdate(AppState::SecretModeInGame)),
        )
        .add_systems(
            (finish_secret_mode, apply_system_buffers)
                .chain()
                .after(Set::PlayerMovement)
                .before(Set::FireSpawn)
                .in_set(OnUpdate(AppState::SecretModeInGame)),
        );
    }
}
