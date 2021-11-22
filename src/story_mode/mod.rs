use bevy::prelude::*;

use crate::{
    game::{
        add_common_game_systems,
        systems::{game_timer_tick, hud_update, resize_window, spawn_cameras},
    },
    AppState, Label,
};

use self::systems::*;

mod constants;
mod resources;
mod systems;
mod utils;

pub struct StoryModePlugin;

impl Plugin for StoryModePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::StoryMode)
                .with_system(setup_story_mode.exclusive_system().label(Label::Setup))
                .with_system(resize_window.exclusive_system().after(Label::Setup))
                .with_system(spawn_cameras.exclusive_system().after(Label::Setup)),
        )
        .add_system_set(SystemSet::on_exit(AppState::StoryMode).with_system(teardown))
        .add_system_set(SystemSet::on_update(AppState::StoryMode).with_system(story_mode_dispatch))
        .add_system_set(SystemSet::on_enter(AppState::BossSpeech).with_system(setup_boss_speech))
        .add_system_set(SystemSet::on_update(AppState::BossSpeech).with_system(boss_speech_update))
        .add_system_set(
            SystemSet::on_enter(AppState::HighScoreNameInput)
                .with_system(setup_high_score_name_input),
        )
        .add_system_set(
            SystemSet::on_update(AppState::HighScoreNameInput)
                .with_system(high_score_name_input_update),
        );

        add_common_game_systems(app, AppState::StoryModeInGame);
        app.add_system_set(
            SystemSet::on_update(AppState::StoryModeInGame)
                .with_system(game_timer_tick.label(Label::TimeUpdate))
                // game end check
                .with_system(
                    finish_level
                        .exclusive_system()
                        .at_end()
                        .label(Label::GameEndCheck),
                )
                // update HUD
                .with_system(
                    hud_update
                        .exclusive_system()
                        .at_end()
                        .after(Label::GameEndCheck),
                )
                .with_system(hud_lives_indicator_update.after(Label::Damage))
                .with_system(hud_points_indicator_update.after(Label::Damage)),
        );
    }
}
