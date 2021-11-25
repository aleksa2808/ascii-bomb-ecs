use bevy::prelude::*;

use crate::AppState;

use self::{
    resources::{MainMenuSoundEffects, MenuMaterials, MenuState},
    systems::*,
};

mod components;
mod constants;
mod resources;
mod systems;
mod utils;

pub use constants::{MENU_HEIGHT, MENU_WIDTH};

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
enum MenuLabel {
    Setup,
    Navigation,
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuMaterials>()
            .init_resource::<MainMenuSoundEffects>()
            .init_resource::<MenuState>()
            .add_system_set(
                SystemSet::on_enter(AppState::MainMenu)
                    .with_system(setup_menu.exclusive_system().label(MenuLabel::Setup))
                    .with_system(resize_window.exclusive_system().after(MenuLabel::Setup)),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::MainMenu).with_system(teardown.exclusive_system()),
            )
            .add_system_set(
                SystemSet::on_update(AppState::MainMenu)
                    .with_system(
                        menu_navigation
                            .exclusive_system()
                            .label(MenuLabel::Navigation),
                    )
                    .with_system(
                        menu_demo_mode_trigger
                            .exclusive_system()
                            .after(MenuLabel::Navigation),
                    )
                    .with_system(animate_menu_background.exclusive_system()),
            );
    }
}
