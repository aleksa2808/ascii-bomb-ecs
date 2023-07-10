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
            (setup_splash_screen, apply_system_buffers)
                .chain()
                .in_schedule(OnEnter(AppState::SplashScreen)),
        )
        .add_system(splash_screen_update.in_set(OnUpdate(AppState::SplashScreen)))
        .add_systems(
            (teardown, apply_system_buffers)
                .chain()
                .in_schedule(OnExit(AppState::SplashScreen)),
        );
    }
}
