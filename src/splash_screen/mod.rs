mod constants;
mod resources;
mod systems;

use bevy::prelude::*;

use crate::AppState;

use self::systems::*;

pub struct SplashScreenPlugin;

impl Plugin for SplashScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::SplashScreen)
                .with_system(setup_splash_screen.exclusive_system()),
        )
        .add_system_set(
            SystemSet::on_update(AppState::SplashScreen).with_system(splash_screen_update),
        )
        .add_system_set(
            SystemSet::on_exit(AppState::SplashScreen).with_system(teardown.exclusive_system()),
        );
    }
}
