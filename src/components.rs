use bevy::prelude::*;

use crate::types::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub y: isize,
    pub x: isize,
}

pub struct Player;

pub struct Moving {
    pub direction: Direction,
    pub step_timer: Timer,
}

pub struct Health {
    pub lives: usize,
    pub max_health: usize,
    pub health: usize,
}

pub struct BaseMaterial(pub Handle<ColorMaterial>);
pub struct ImmortalMaterial(pub Handle<ColorMaterial>);

pub struct Bomb {
    pub parent: Entity,
    pub range: usize,
}

pub struct Fuse {}

pub struct BombSatchel {
    pub bombs_available: usize,
    pub bomb_range: usize,
}

pub struct Immortal {
    pub timer: Timer,
}

pub struct WallHack;

pub struct BombPush;

pub struct MeleeAttacker;

pub struct TeamAlignment(pub usize);

pub struct Perishable {
    pub timer: Timer,
}

pub struct Fire;

pub struct Solid;

pub struct Wall;

pub struct Destructible;

// Bundles

#[derive(Bundle)]
pub struct ImmortalBundle {
    pub immortal: Immortal,
    pub animation_timer: Timer,
}

impl Default for ImmortalBundle {
    fn default() -> Self {
        ImmortalBundle {
            immortal: Immortal {
                timer: Timer::from_seconds(2.0, false),
            },
            animation_timer: Timer::from_seconds(0.66, true),
        }
    }
}
