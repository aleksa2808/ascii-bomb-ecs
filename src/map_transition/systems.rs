use bevy::prelude::*;

use crate::{
    game::components::{Player, Wall},
    AppState,
};

use super::resources::{MapTransitionContext, MapTransitionInput};

pub fn setup_map_transition(
    mut commands: Commands,
    mut map_spawn_input: ResMut<MapTransitionInput>,
    mut query: Query<&mut Visibility, Or<(With<Wall>, With<Player>)>>,
) {
    // hide wall and player entities
    for mut visible in query.iter_mut() {
        *visible = Visibility::Hidden;
    }

    commands.insert_resource(MapTransitionContext {
        wall_entity_reveal_groups: map_spawn_input
            .wall_entity_reveal_groups
            .drain(..)
            .collect(),
        reveal_timer: Timer::from_seconds(0.015, TimerMode::Repeating),
        next_state: map_spawn_input.next_state,
    });
    commands.remove_resource::<MapTransitionInput>();
}

pub fn map_transition_update(
    time: Res<Time>,
    mut map_transition_context: ResMut<MapTransitionContext>,
    mut next_state: ResMut<NextState<AppState>>,
    mut query: Query<&mut Visibility>,
) {
    // TODO: why is the first tick much larger? it progresses the transition further than we want
    map_transition_context.reveal_timer.tick(time.delta());
    for _ in 0..map_transition_context
        .reveal_timer
        .times_finished_this_tick()
    {
        if let Some(reveal_group) = map_transition_context.wall_entity_reveal_groups.pop_front() {
            for entity in reveal_group {
                *query.get_mut(entity).unwrap() = Visibility::Inherited;
            }
        } else {
            // reveal the rest of the hidden entities
            for mut visible in query.iter_mut() {
                *visible = Visibility::Inherited;
            }
            next_state.set(map_transition_context.next_state);
            break;
        }
    }
}

pub fn teardown(mut commands: Commands) {
    commands.remove_resource::<MapTransitionContext>();
}
