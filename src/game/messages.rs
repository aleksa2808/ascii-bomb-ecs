use bevy::{ecs as bevy_ecs, prelude::*};

use super::components::*;
use super::types::*;

#[derive(Message)]
pub struct PlayerActionMessage {
    pub player: Entity,
    pub action: PlayerAction,
}

#[derive(Clone, Copy, Message)]
pub struct ExplosionMessage {
    pub bomb: Entity,
}

#[derive(Message)]
pub struct BombRestockMessage {
    pub satchel_owner: Entity,
}

#[derive(Message)]
pub struct DamageMessage {
    pub target: Entity,
}

#[derive(Message)]
pub struct BurnMessage {
    pub position: Position,
}

#[derive(Message)]
pub struct PlayerDeathMessage;
