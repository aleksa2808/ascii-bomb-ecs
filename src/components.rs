use bevy::prelude::*;

use crate::types::{Cooldown, Direction};

// UI

pub struct UIRoot;

pub struct UIComponent;

// menu components

pub struct Cursor;

pub struct MenuOptionText;

// game components

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub y: isize,
    pub x: isize,
}

impl Position {
    pub fn offset(&self, direction: Direction, distance: usize) -> Self {
        let distance = distance as isize;

        let (y_offset, x_offset) = match direction {
            Direction::Right => (0, distance),
            Direction::Down => (distance, 0),
            Direction::Left => (0, -distance),
            Direction::Up => (-distance, 0),
        };

        Position {
            y: self.y + y_offset,
            x: self.x + x_offset,
        }
    }
}

pub struct Player;

pub struct Protagonist;

// usize represents the penguin's ID
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Penguin(pub usize);

// usize represents the controller/player number
pub struct HumanControlled(pub usize);

#[derive(Default)]
pub struct MobAI {
    pub direction: Option<Direction>,
}

pub struct BotAI;

pub struct MoveCooldown(pub Cooldown);

// currently used for bomb movement only, will stop before items
pub struct Moving {
    pub direction: Direction,
}

pub struct Health {
    pub lives: usize,
    pub max_health: usize,
    pub health: usize,
}

pub struct BaseMaterial(pub Handle<ColorMaterial>);
pub struct ImmortalMaterial(pub Handle<ColorMaterial>);

#[derive(Clone, Copy)]
pub struct Bomb {
    pub parent: Entity,
    pub range: usize,
}

pub struct Fuse;

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

pub struct TeamID(pub usize);

pub struct PointValue(pub usize);

pub struct Perishable {
    pub timer: Timer,
}

pub struct Fire;

pub struct Solid;

pub struct Wall;

pub struct Destructible;

pub struct Exit {
    pub spawn_cooldown: Cooldown,
}

impl Default for Exit {
    fn default() -> Self {
        Exit {
            spawn_cooldown: Cooldown::from_seconds(2.0),
        }
    }
}

// HUD display

pub struct HUDRoot;

pub struct GameTimerDisplay;

pub struct LivesDisplay;

pub struct PointsDisplay;

pub struct PenguinPortraitDisplay;

pub struct PenguinPortrait(pub Penguin);

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
