use bevy::prelude::*;

use super::components::*;
use super::types::*;

pub struct PlayerActionEvent(pub Entity, pub PlayerAction);

// position + range
#[derive(Clone, Copy)]
pub struct ExplosionEvent(pub Position, pub usize);

pub struct DamageEvent(pub Entity);

pub struct BurnEvent(pub Position);
