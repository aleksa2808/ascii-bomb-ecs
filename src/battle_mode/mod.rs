mod constants;
mod resources;
mod systems;
mod utils;

use bevy::prelude::*;

use crate::{
    game::{
        add_common_game_systems,
        systems::{game_timer_tick, hud_update, resize_window, spawn_cameras},
        Label,
    },
    AppState,
};

pub use self::resources::BattleModeConfiguration;
use self::systems::*;

pub struct BattleModePlugin;

impl Plugin for BattleModePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::BattleMode)
                .with_system(setup_battle_mode.exclusive_system().label(Label::Setup))
                .with_system(resize_window.exclusive_system().after(Label::Setup))
                .with_system(spawn_cameras.exclusive_system().after(Label::Setup)),
        )
        .add_system_set(
            SystemSet::on_exit(AppState::BattleMode).with_system(teardown.exclusive_system()),
        )
        .add_system_set(
            SystemSet::on_update(AppState::BattleMode)
                .with_system(battle_mode_dispatch.exclusive_system()),
        )
        .add_system_set(
            SystemSet::on_update(AppState::RoundStartFreeze)
                .with_system(finish_freeze.exclusive_system()),
        )
        .add_system_set(
            SystemSet::on_enter(AppState::LeaderboardDisplay)
                .with_system(setup_leaderboard_display.exclusive_system()),
        )
        .add_system_set(
            SystemSet::on_update(AppState::LeaderboardDisplay)
                .with_system(leaderboard_display_update.exclusive_system()),
        );

        add_common_game_systems(app, AppState::BattleModeInGame);
        app.add_system_set(
            SystemSet::on_enter(AppState::BattleModeInGame)
                .with_system(trigger_round_start_freeze.exclusive_system()),
        )
        .add_system_set(
            SystemSet::on_update(AppState::BattleModeInGame)
                .with_system(game_timer_tick.exclusive_system().label(Label::TimeUpdate))
                .with_system(
                    wall_of_death_update
                        .exclusive_system()
                        .label(Label::PlayerDeathEvent)
                        .label(Label::BombRestockEvent),
                )
                .with_system(
                    on_death_item_pinata
                        .exclusive_system()
                        .label(Label::ItemSpawn)
                        .after(Label::PlayerDeathEvent),
                )
                .with_system(
                    finish_round
                        .exclusive_system()
                        .after(Label::TimeUpdate)
                        .after(Label::PlayerDeathEvent),
                )
                // update HUD
                .with_system(
                    hud_update
                        .exclusive_system()
                        .after(Label::TimeUpdate)
                        .after(Label::PlayerDeathEvent),
                ),
        );
    }
}
