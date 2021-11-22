use bevy::prelude::{App, Plugin, SystemSet};

use crate::AppState;

use self::systems::{map_transition_update, setup_map_transition, teardown};

mod resources;
mod systems;

pub use self::resources::MapTransitionInput;

pub struct MapTransitionPlugin;

impl Plugin for MapTransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::MapTransition).with_system(setup_map_transition),
        )
        .add_system_set(
            SystemSet::on_update(AppState::MapTransition).with_system(map_transition_update),
        )
        .add_system_set(SystemSet::on_exit(AppState::MapTransition).with_system(teardown));
    }
}
