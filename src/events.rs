use bevy::prelude::*;

use crate::components::*;
use crate::types::*;

pub struct PlayerActionEvent(pub Entity, pub PlayerAction);

// position + range
#[derive(Clone, Copy)]
pub struct ExplosionEvent(pub Position, pub usize);

pub struct DamageEvent(pub Entity);

pub struct BurnEvent(pub Position);
