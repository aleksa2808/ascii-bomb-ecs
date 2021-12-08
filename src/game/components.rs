use bevy::{ecs as bevy_ecs, prelude::*};

use super::types::{BotDifficulty, Cooldown, Direction, Power, Upgrade};

#[derive(Component)]
pub struct ColorComponent(pub Color);

// UI

#[derive(Component)]
pub struct UIRoot;

#[derive(Component)]
pub struct UIComponent;

// game components

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
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

#[derive(Component)]
pub struct SpawnPosition(pub Position);

#[derive(Component)]
pub struct Player;

// usize represents the penguin's ID
#[derive(Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct Penguin(pub usize);

// usize represents the controller/player number
#[derive(Component)]
pub struct HumanControlled(pub usize);

#[derive(Default, Component)]
pub struct MobAI {
    pub direction: Option<Direction>,
}

#[derive(Component)]
pub struct BotAI {
    pub difficulty: BotDifficulty,
}

#[derive(Component)]
pub struct MoveCooldown(pub Cooldown);

// currently used for bomb movement only, will stop before items
#[derive(Component)]
pub struct Moving {
    pub direction: Direction,
}

#[derive(Component)]
pub struct Health {
    pub lives: usize,
    pub max_health: usize,
    pub health: usize,
}

#[derive(Component)]
pub struct BaseMaterial(pub Handle<ColorMaterial>);
#[derive(Component)]
pub struct ImmortalMaterial(pub Handle<ColorMaterial>);

#[derive(Clone, Component)]
pub struct Bomb {
    pub owner: Option<Entity>,
    pub range: usize,
    pub timer: Timer,
}

#[derive(Component)]
pub struct Fuse;

#[derive(Component)]
pub struct BombSatchel {
    pub bombs_available: usize,
    pub bomb_range: usize,
}

#[derive(Component)]
pub struct Immortal {
    pub timer: Timer,
}

#[derive(Component)]
pub struct WallHack;

#[derive(Component)]
pub struct BombPush;

#[derive(Component)]
pub struct MeleeAttacker;

#[derive(Component)]
pub struct TeamID(pub usize);

#[derive(Component)]
pub struct PointValue(pub usize);

#[derive(Component)]
pub struct Fire {
    pub timer: Timer,
}

#[derive(Component)]
pub struct Solid;

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct Destructible;

#[derive(Component)]
pub struct Crumbling {
    pub timer: Timer,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum Item {
    Upgrade(Upgrade),
    Power(Power),
}

#[derive(Component)]
pub struct BurningItem {
    pub timer: Timer,
}

#[derive(Component)]
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

#[derive(Component)]
pub struct HUDRoot;

#[derive(Component)]
pub struct GameTimerDisplay;

#[derive(Component)]
pub struct BottomLeftDisplay1;

#[derive(Component)]
pub struct BottomLeftDisplay2;

#[derive(Component)]
pub struct PenguinPortraitDisplay;

#[derive(Component)]
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
