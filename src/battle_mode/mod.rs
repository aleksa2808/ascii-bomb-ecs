mod constants;
mod resources;
mod systems;
mod types;
mod utils;

use bevy::prelude::*;

use crate::{
    game::{
        add_common_game_systems,
        systems::{
            game_timer_tick, hud_update, resize_window, setup_penguin_portraits, spawn_cameras,
            wall_of_death_update,
        },
        Set,
    },
    AppState,
};

pub use self::resources::BattleModeConfiguration;
use self::{resources::LeaderboardTextures, systems::*};

pub struct BattleModePlugin;

impl Plugin for BattleModePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LeaderboardTextures>()
            .add_systems(
                (setup_battle_mode, apply_system_buffers)
                    .chain()
                    .in_set(Set::Setup)
                    .in_schedule(OnEnter(AppState::BattleModeSetup)),
            )
            .add_systems(
                (resize_window, apply_system_buffers)
                    .chain()
                    .after(Set::Setup)
                    .in_schedule(OnEnter(AppState::BattleModeSetup)),
            )
            .add_systems(
                (spawn_cameras, apply_system_buffers)
                    .chain()
                    .after(Set::Setup)
                    .in_schedule(OnEnter(AppState::BattleModeSetup)),
            )
            .add_systems(
                (teardown, apply_system_buffers)
                    .chain()
                    .in_schedule(OnEnter(AppState::BattleModeTeardown)),
            )
            .add_systems(
                (battle_mode_manager, apply_system_buffers)
                    .chain()
                    .in_set(OnUpdate(AppState::BattleModeManager)),
            )
            .add_systems(
                (setup_penguin_portraits, apply_system_buffers)
                    .chain()
                    .in_schedule(OnEnter(AppState::RoundStartFreeze)),
            )
            .add_systems(
                (finish_freeze, apply_system_buffers)
                    .chain()
                    .in_set(OnUpdate(AppState::RoundStartFreeze)),
            )
            .add_systems(
                (setup_leaderboard_display, apply_system_buffers)
                    .chain()
                    .in_schedule(OnEnter(AppState::LeaderboardDisplay)),
            )
            .add_systems(
                (leaderboard_display_update, apply_system_buffers)
                    .chain()
                    .in_set(OnUpdate(AppState::LeaderboardDisplay)),
            );

        add_common_game_systems(app, AppState::BattleModeInGame);
        app.add_systems(
            (game_timer_tick, apply_system_buffers)
                .chain()
                .in_set(Set::TimeUpdate)
                .in_set(OnUpdate(AppState::BattleModeInGame)),
        )
        .add_systems(
            (wall_of_death_update, apply_system_buffers)
                .chain()
                .in_set(Set::PlayerDeathEvent)
                .in_set(Set::BombRestockEvent)
                .in_set(OnUpdate(AppState::BattleModeInGame)),
        )
        .add_systems(
            (on_death_item_pinata, apply_system_buffers)
                .chain()
                .in_set(Set::ItemSpawn)
                .after(Set::PlayerDeathEvent)
                .in_set(OnUpdate(AppState::BattleModeInGame)),
        )
        .add_systems(
            (finish_round, apply_system_buffers)
                .chain()
                .after(Set::TimeUpdate)
                .after(Set::PlayerDeathEvent)
                .in_set(OnUpdate(AppState::BattleModeInGame)),
        )
        // update HUD
        .add_systems(
            (hud_update, apply_system_buffers)
                .chain()
                .after(Set::TimeUpdate)
                .after(Set::PlayerDeathEvent)
                .in_set(OnUpdate(AppState::BattleModeInGame)),
        );
    }
}
