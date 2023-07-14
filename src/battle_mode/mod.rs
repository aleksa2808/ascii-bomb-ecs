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
                OnEnter(AppState::BattleModeSetup),
                (setup_battle_mode, apply_deferred)
                    .chain()
                    .in_set(Set::Setup),
            )
            .add_systems(
                OnEnter(AppState::BattleModeSetup),
                (resize_window, apply_deferred).chain().after(Set::Setup),
            )
            .add_systems(
                OnEnter(AppState::BattleModeSetup),
                (spawn_cameras, apply_deferred).chain().after(Set::Setup),
            )
            .add_systems(
                OnEnter(AppState::BattleModeTeardown),
                (teardown, apply_deferred).chain(),
            )
            .add_systems(
                Update,
                (battle_mode_manager, apply_deferred)
                    .chain()
                    .run_if(in_state(AppState::BattleModeManager)),
            )
            .add_systems(
                OnEnter(AppState::RoundStartFreeze),
                (setup_penguin_portraits, apply_deferred).chain(),
            )
            .add_systems(
                Update,
                (finish_freeze, apply_deferred)
                    .chain()
                    .run_if(in_state(AppState::RoundStartFreeze)),
            )
            .add_systems(
                OnEnter(AppState::LeaderboardDisplay),
                (setup_leaderboard_display, apply_deferred).chain(),
            )
            .add_systems(
                Update,
                (leaderboard_display_update, apply_deferred)
                    .chain()
                    .run_if(in_state(AppState::LeaderboardDisplay)),
            );

        add_common_game_systems(app, AppState::BattleModeInGame);
        app.add_systems(
            Update,
            (game_timer_tick, apply_deferred)
                .chain()
                .in_set(Set::TimeUpdate)
                .run_if(in_state(AppState::BattleModeInGame)),
        )
        .add_systems(
            Update,
            (wall_of_death_update, apply_deferred)
                .chain()
                .in_set(Set::PlayerDeathEvent)
                .in_set(Set::BombRestockEvent)
                .run_if(in_state(AppState::BattleModeInGame)),
        )
        .add_systems(
            Update,
            (on_death_item_pinata, apply_deferred)
                .chain()
                .in_set(Set::ItemSpawn)
                .after(Set::PlayerDeathEvent)
                .run_if(in_state(AppState::BattleModeInGame)),
        )
        .add_systems(
            Update,
            (finish_round, apply_deferred)
                .chain()
                .after(Set::TimeUpdate)
                .after(Set::PlayerDeathEvent)
                .run_if(in_state(AppState::BattleModeInGame)),
        )
        // update HUD
        .add_systems(
            Update,
            (hud_update, apply_deferred)
                .chain()
                .after(Set::TimeUpdate)
                .after(Set::PlayerDeathEvent)
                .run_if(in_state(AppState::BattleModeInGame)),
        );
    }
}
