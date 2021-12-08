use bevy::prelude::*;

use super::components::*;
use super::types::*;

pub struct PlayerActionEvent {
    pub player: Entity,
    pub action: PlayerAction,
}

#[derive(Clone, Copy)]
pub struct ExplosionEvent {
    pub bomb: Entity,
}

pub struct BombRestockEvent {
    pub satchel_owner: Entity,
}

pub struct DamageEvent {
    pub target: Entity,
}

pub struct BurnEvent {
    pub position: Position,
}

pub struct PlayerDeathEvent;
