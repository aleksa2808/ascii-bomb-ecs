use bevy::{ecs as bevy_ecs, prelude::*};

use super::components::*;
use super::types::*;

#[derive(Event)]
pub struct PlayerActionEvent {
    pub player: Entity,
    pub action: PlayerAction,
}

#[derive(Clone, Copy, Event)]
pub struct ExplosionEvent {
    pub bomb: Entity,
}

#[derive(Event)]
pub struct BombRestockEvent {
    pub satchel_owner: Entity,
}

#[derive(Event)]
pub struct DamageEvent {
    pub target: Entity,
}

#[derive(Event)]
pub struct BurnEvent {
    pub position: Position,
}

#[derive(Event)]
pub struct PlayerDeathEvent;
