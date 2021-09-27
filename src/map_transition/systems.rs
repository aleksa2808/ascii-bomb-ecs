use bevy::prelude::*;

use crate::{
    components::{Player, Wall},
    AppState,
};

use super::resources::{MapTransitionContext, MapTransitionInput};

pub fn setup_map_transition(
    mut commands: Commands,
    mut map_spawn_input: ResMut<MapTransitionInput>,
    mut query: Query<&mut Visible, Or<(With<Wall>, With<Player>)>>,
) {
    // hide wall and player entities
    for mut visible in query.iter_mut() {
        visible.is_visible = false;
    }

    commands.insert_resource(MapTransitionContext {
        wall_entity_reveal_groups: map_spawn_input
            .wall_entity_reveal_groups
            .drain(..)
            .collect(),
        reveal_timer: Timer::from_seconds(0.015, true),
    });
    commands.remove_resource::<MapTransitionInput>();
}

pub fn map_transition_update(
    time: Res<Time>,
    mut map_transition_context: ResMut<MapTransitionContext>,
    mut state: ResMut<State<AppState>>,
    mut query: Query<&mut Visible>,
) {
    // TODO: why is the first tick much larger? it progresses the transition further than we want
    map_transition_context.reveal_timer.tick(time.delta());
    for _ in 0..map_transition_context.reveal_timer.times_finished() {
        if let Some(reveal_group) = map_transition_context.wall_entity_reveal_groups.pop_front() {
            for entity in reveal_group {
                query.get_mut(entity).unwrap().is_visible = true;
            }
        } else {
            // reveal the rest of the hidden entities
            for mut visible in query.iter_mut() {
                visible.is_visible = true;
            }
            state.pop().unwrap();
            break;
        }
    }
}

pub fn teardown(mut commands: Commands) {
    commands.remove_resource::<MapTransitionContext>();
}
