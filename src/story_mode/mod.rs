use bevy::prelude::*;

use crate::{
    common::systems::clear_inputs,
    game::{
        add_common_game_systems,
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
            (setup_story_mode, apply_system_buffers)
                .chain()
                .in_set(Set::Setup)
                .in_schedule(OnEnter(AppState::StoryModeSetup)),
        )
        .add_systems(
            (resize_window, apply_system_buffers)
                .chain()
                .after(Set::Setup)
                .in_schedule(OnEnter(AppState::StoryModeSetup)),
        )
        .add_systems(
            (spawn_cameras, apply_system_buffers)
                .chain()
                .after(Set::Setup)
                .in_schedule(OnEnter(AppState::StoryModeSetup)),
        )
        .add_systems(
            (teardown, apply_system_buffers)
                .chain()
                .in_schedule(OnEnter(AppState::StoryModeTeardown)),
        )
        .add_systems(
            (story_mode_manager, apply_system_buffers)
                .chain()
                .in_set(OnUpdate(AppState::StoryModeManager)),
        )
        .add_systems(
            (setup_boss_speech, apply_system_buffers)
                .chain()
                .in_schedule(OnEnter(AppState::BossSpeech)),
        )
        .add_systems(
            (boss_speech_update, apply_system_buffers)
                .chain()
                .after(crate::common::Label::InputMapping)
                .in_set(OnUpdate(AppState::BossSpeech)),
        )
        .add_systems(
            (setup_high_score_name_input, apply_system_buffers)
                .chain()
                .in_schedule(OnEnter(AppState::HighScoreNameInput)),
        )
        .add_systems(
            (high_score_name_input_update, apply_system_buffers)
                .chain()
                .after(crate::common::Label::InputMapping)
                .in_set(OnUpdate(AppState::HighScoreNameInput)),
        );

        app.add_systems(
            (setup_penguin_portraits, apply_system_buffers)
                .chain()
                .in_schedule(OnEnter(AppState::StoryModeInGame)),
        );
        add_common_game_systems(app, AppState::StoryModeInGame);
        app.add_system(clear_inputs.in_schedule(OnExit(AppState::StoryModeInGame)))
            .add_systems(
                (game_timer_tick, apply_system_buffers)
                    .chain()
                    .in_set(Set::TimeUpdate)
                    .in_set(OnUpdate(AppState::StoryModeInGame)),
            )
            // game end check
            .add_systems(
                (finish_level, apply_system_buffers)
                    .chain()
                    .after(Set::TimeUpdate)
                    .after(Set::PlayerMovement)
                    .after(Set::PlayerDeathEvent)
                    .in_set(OnUpdate(AppState::StoryModeInGame)),
            )
            // update HUD
            .add_systems(
                (hud_update, apply_system_buffers)
                    .chain()
                    .after(Set::TimeUpdate)
                    .after(Set::PlayerDeathEvent)
                    .in_set(OnUpdate(AppState::StoryModeInGame)),
            )
            .add_systems(
                (hud_lives_indicator_update, apply_system_buffers)
                    .chain()
                    .after(Set::DamageApplication)
                    .in_set(OnUpdate(AppState::StoryModeInGame)),
            )
            .add_systems(
                (hud_points_indicator_update, apply_system_buffers)
                    .chain()
                    .after(Set::PlayerDeathEvent)
                    .in_set(OnUpdate(AppState::StoryModeInGame)),
            );
    }
}
