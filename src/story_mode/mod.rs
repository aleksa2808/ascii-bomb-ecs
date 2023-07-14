use bevy::prelude::*;

use crate::{
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
            OnEnter(AppState::StoryModeSetup),
            (setup_story_mode, apply_deferred)
                .chain()
                .in_set(Set::Setup),
        )
        .add_systems(
            OnEnter(AppState::StoryModeSetup),
            (resize_window, apply_deferred).chain().after(Set::Setup),
        )
        .add_systems(
            OnEnter(AppState::StoryModeSetup),
            (spawn_cameras, apply_deferred).chain().after(Set::Setup),
        )
        .add_systems(
            OnEnter(AppState::StoryModeTeardown),
            (teardown, apply_deferred).chain(),
        )
        .add_systems(
            Update,
            (story_mode_manager, apply_deferred)
                .chain()
                .run_if(in_state(AppState::StoryModeManager)),
        )
        .add_systems(
            OnEnter(AppState::BossSpeech),
            (setup_boss_speech, apply_deferred).chain(),
        )
        .add_systems(
            Update,
            (boss_speech_update, apply_deferred)
                .chain()
                .after(crate::common::Label::InputMapping)
                .run_if(in_state(AppState::BossSpeech)),
        )
        .add_systems(
            OnEnter(AppState::HighScoreNameInput),
            (setup_high_score_name_input, apply_deferred).chain(),
        )
        .add_systems(
            Update,
            (high_score_name_input_update, apply_deferred)
                .chain()
                .after(crate::common::Label::InputMapping)
                .run_if(in_state(AppState::HighScoreNameInput)),
        );

        app.add_systems(
            OnEnter(AppState::StoryModeInGame),
            (setup_penguin_portraits, apply_deferred).chain(),
        );
        add_common_game_systems(app, AppState::StoryModeInGame);
        app.add_systems(
            Update,
            (game_timer_tick, apply_deferred)
                .chain()
                .in_set(Set::TimeUpdate)
                .run_if(in_state(AppState::StoryModeInGame)),
        )
        // game end check
        .add_systems(
            Update,
            (finish_level, apply_deferred)
                .chain()
                .after(Set::TimeUpdate)
                .after(Set::PlayerMovement)
                .after(Set::PlayerDeathEvent)
                .run_if(in_state(AppState::StoryModeInGame)),
        )
        // update HUD
        .add_systems(
            Update,
            (hud_update, apply_deferred)
                .chain()
                .after(Set::TimeUpdate)
                .after(Set::PlayerDeathEvent)
                .run_if(in_state(AppState::StoryModeInGame)),
        )
        .add_systems(
            Update,
            (hud_lives_indicator_update, apply_deferred)
                .chain()
                .after(Set::DamageApplication)
                .run_if(in_state(AppState::StoryModeInGame)),
        )
        .add_systems(
            Update,
            (hud_points_indicator_update, apply_deferred)
                .chain()
                .after(Set::PlayerDeathEvent)
                .run_if(in_state(AppState::StoryModeInGame)),
        );
    }
}
