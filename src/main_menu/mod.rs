use bevy::{ecs as bevy_ecs, prelude::*};

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

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
enum MenuLabel {
    Setup,
    Navigation,
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuColors>()
            .init_resource::<MainMenuSoundEffects>()
            .init_resource::<MenuState>()
            .add_systems(
                (setup_menu, apply_system_buffers)
                    .chain()
                    .in_set(MenuLabel::Setup)
                    .in_schedule(OnEnter(AppState::MainMenu)),
            )
            .add_systems(
                (resize_window, apply_system_buffers)
                    .chain()
                    .after(MenuLabel::Setup)
                    .in_schedule(OnEnter(AppState::MainMenu)),
            )
            .add_systems(
                (teardown, apply_system_buffers)
                    .chain()
                    .in_schedule(OnExit(AppState::MainMenu)),
            )
            .add_systems(
                (menu_navigation, apply_system_buffers)
                    .chain()
                    .in_set(MenuLabel::Navigation)
                    .after(crate::common::Label::InputMapping)
                    .in_set(OnUpdate(AppState::MainMenu)),
            )
            .add_systems(
                (menu_demo_mode_trigger, apply_system_buffers)
                    .chain()
                    .after(MenuLabel::Navigation)
                    .after(crate::common::Label::InputMapping),
            )
            .add_systems(
                (animate_menu_background, apply_system_buffers)
                    .chain()
                    .in_set(OnUpdate(AppState::MainMenu)),
            );
    }
}
