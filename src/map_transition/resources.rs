use std::collections::VecDeque;

use bevy::{
    ecs as bevy_ecs,
    prelude::{Entity, Resource},
    time::Timer,
};

#[derive(Resource)]
pub struct MapTransitionInput {
    pub wall_entity_reveal_groups: Vec<Vec<Entity>>,
}

#[derive(Resource)]
pub struct MapTransitionContext {
    pub wall_entity_reveal_groups: VecDeque<Vec<Entity>>,
    pub reveal_timer: Timer,
}
