use bevy::prelude::*;

use crate::{
    game::{
        common_game_systems,
        systems::{
            game_timer_tick, hud_update, resize_window, setup_penguin_portraits, spawn_cameras,
        },
        Set,
    },
    AppState,
};

use self::systems::*;

mod components;
mod constants;
mod resources;
mod systems;
mod utils;

pub struct StoryModePlugin;

impl Plugin for StoryModePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::StoryModeSetup),
            (
                setup_story_mode,
                apply_deferred,
                (resize_window, spawn_cameras),
            )
                .chain(),
        )
        .add_systems(OnEnter(AppState::StoryModeTeardown), teardown)
        .add_systems(
            Update,
            story_mode_manager.run_if(in_state(AppState::StoryModeManager)),
        )
        .add_systems(OnEnter(AppState::BossSpeech), setup_boss_speech)
        .add_systems(
            Update,
            boss_speech_update
                .after(crate::common::Label::InputMapping)
                .run_if(in_state(AppState::BossSpeech)),
        )
        .add_systems(
            OnEnter(AppState::HighScoreNameInput),
            setup_high_score_name_input,
        )
        .add_systems(
            Update,
            high_score_name_input_update
                .after(crate::common::Label::InputMapping)
                .run_if(in_state(AppState::HighScoreNameInput)),
        );

        app.add_systems(OnEnter(AppState::StoryModeInGame), setup_penguin_portraits);
        app.add_systems(
            Update,
            (
                common_game_systems(),
                game_timer_tick.in_set(Set::TimeUpdate),
                // game end check
                finish_level
                    .after(Set::TimeUpdate)
                    .after(Set::PlayerMovement)
                    .after(Set::PlayerDeathEvent),
                // update HUD
                hud_update
                    .after(Set::TimeUpdate)
                    .after(Set::PlayerDeathEvent),
                hud_lives_indicator_update.after(Set::DamageApplication),
                hud_points_indicator_update.after(Set::PlayerDeathEvent),
            )
                .run_if(in_state(AppState::StoryModeInGame)),
        );
    }
}
