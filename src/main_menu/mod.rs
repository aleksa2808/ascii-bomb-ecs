use bevy::prelude::*;

use crate::AppState;

use self::{
    resources::{MainMenuSoundEffects, MenuColors, MenuState},
    systems::*,
};

mod components;
mod constants;
mod resources;
mod systems;
mod utils;

pub use constants::{MENU_HEIGHT, MENU_WIDTH};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuColors>()
            .init_resource::<MainMenuSoundEffects>()
            .init_resource::<MenuState>()
            .add_systems(
                OnEnter(AppState::MainMenu),
                (
                    (setup_menu, apply_deferred).chain(),
                    (resize_window, apply_deferred).chain(),
                )
                    .chain(),
            )
            .add_systems(
                OnExit(AppState::MainMenu),
                (teardown, apply_deferred).chain(),
            )
            .add_systems(
                Update,
                (
                    (
                        (menu_navigation, apply_deferred).chain(),
                        (menu_demo_mode_trigger, apply_deferred).chain(),
                    )
                        .chain()
                        .after(crate::common::Label::InputMapping),
                    (animate_menu_background, apply_deferred).chain(),
                )
                    .run_if(in_state(AppState::MainMenu)),
            );
    }
}
