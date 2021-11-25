use bevy::prelude::*;

use crate::{
    game::{
        add_common_game_systems,
        systems::{resize_window, spawn_cameras},
        Label,
    },
    AppState,
};

use self::{resources::SecretModeMusic, systems::*};

mod resources;
mod systems;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
enum SecretModeLabel {
    Setup,
}

pub struct SecretModePlugin;

impl Plugin for SecretModePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SecretModeMusic>()
            .add_system_set(
                SystemSet::on_enter(AppState::SecretMode)
                    .with_system(
                        setup_secret_mode
                            .exclusive_system()
                            .label(SecretModeLabel::Setup),
                    )
                    .with_system(
                        resize_window
                            .exclusive_system()
                            .after(SecretModeLabel::Setup),
                    )
                    .with_system(
                        spawn_cameras
                            .exclusive_system()
                            .after(SecretModeLabel::Setup),
                    ),
            )
            .add_system_set(
                SystemSet::on_update(AppState::SecretMode)
                    .with_system(secret_mode_dispatch.exclusive_system()),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::SecretMode).with_system(teardown.exclusive_system()),
            );

        add_common_game_systems(app, AppState::SecretModeInGame);
        app.add_system_set(
            SystemSet::on_update(AppState::SecretModeInGame)
                .with_system(update_secret_mode.exclusive_system().at_start())
                .with_system(
                    finish_secret_mode
                        .exclusive_system()
                        .after(Label::PlayerMovement)
                        .before(Label::FireSpawn),
                ),
        );
    }
}
