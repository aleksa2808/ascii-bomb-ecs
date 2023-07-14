mod constants;
mod resources;
mod systems;

use bevy::prelude::*;

use crate::AppState;

use self::systems::*;

pub struct SplashScreenPlugin;

impl Plugin for SplashScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::SplashScreen),
            (setup_splash_screen, apply_deferred).chain(),
        )
        .add_systems(
            Update,
            splash_screen_update.run_if(in_state(AppState::SplashScreen)),
        )
        .add_systems(
            OnExit(AppState::SplashScreen),
            (teardown, apply_deferred).chain(),
        );
    }
}
