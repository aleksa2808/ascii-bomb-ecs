use bevy::prelude::*;

use crate::AppState;

use self::systems::{map_transition_update, setup_map_transition, teardown};

mod resources;
mod systems;

pub use self::resources::MapTransitionInput;

pub struct MapTransitionPlugin;

impl Plugin for MapTransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MapTransition), setup_map_transition)
            .add_systems(
                Update,
                map_transition_update.run_if(in_state(AppState::MapTransition)),
            )
            .add_systems(OnExit(AppState::MapTransition), teardown);
    }
}
