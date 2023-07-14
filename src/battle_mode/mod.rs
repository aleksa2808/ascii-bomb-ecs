mod constants;
mod resources;
mod systems;
mod types;
mod utils;

use bevy::prelude::*;

use crate::{
    game::{
        common_game_systems,
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
                (
                    setup_battle_mode,
                    apply_deferred,
                    (resize_window, spawn_cameras),
                )
                    .chain(),
            )
            .add_systems(OnEnter(AppState::BattleModeTeardown), teardown)
            .add_systems(
                Update,
                battle_mode_manager.run_if(in_state(AppState::BattleModeManager)),
            )
            .add_systems(OnEnter(AppState::RoundStartFreeze), setup_penguin_portraits)
            .add_systems(
                Update,
                finish_freeze.run_if(in_state(AppState::RoundStartFreeze)),
            )
            .add_systems(
                OnEnter(AppState::LeaderboardDisplay),
                setup_leaderboard_display,
            )
            .add_systems(
                Update,
                leaderboard_display_update.run_if(in_state(AppState::LeaderboardDisplay)),
            );

        app.add_systems(
            Update,
            (
                common_game_systems(),
                game_timer_tick.in_set(Set::TimeUpdate),
                (wall_of_death_update, apply_deferred)
                    .chain()
                    .in_set(Set::PlayerDeathEvent)
                    .in_set(Set::BombRestockEvent)
                    .in_set(Set::ItemDespawn),
                (
                    (on_death_item_pinata, apply_deferred)
                        .chain()
                        .in_set(Set::ItemSpawn),
                    (finish_round, hud_update).after(Set::TimeUpdate),
                )
                    .after(Set::PlayerDeathEvent),
            )
                .run_if(in_state(AppState::BattleModeInGame)),
        );
    }
}
