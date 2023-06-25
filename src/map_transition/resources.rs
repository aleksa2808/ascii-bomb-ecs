use std::collections::VecDeque;

use bevy::{prelude::Entity, time::Timer};

pub struct MapTransitionInput {
    pub wall_entity_reveal_groups: Vec<Vec<Entity>>,
}

pub struct MapTransitionContext {
    pub wall_entity_reveal_groups: VecDeque<Vec<Entity>>,
    pub reveal_timer: Timer,
}
