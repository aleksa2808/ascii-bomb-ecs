use bevy::prelude::*;

use crate::AppState;

use self::systems::{map_transition_update, setup_map_transition, teardown};

mod resources;
mod systems;

pub use self::resources::MapTransitionInput;

pub struct MapTransitionPlugin;

impl Plugin for MapTransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (setup_map_transition, apply_system_buffers)
                .chain()
                .in_schedule(OnEnter(AppState::MapTransition)),
        )
        .add_system(map_transition_update.in_set(OnUpdate(AppState::MapTransition)))
        .add_systems(
            (teardown, apply_system_buffers)
                .chain()
                .in_schedule(OnExit(AppState::MapTransition)),
        );
    }
}
