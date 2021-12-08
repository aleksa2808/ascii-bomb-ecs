use std::time::Duration;

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub const LIST: [Direction; 4] = [
        Direction::Right,
        Direction::Left,
        Direction::Up,
        Direction::Down,
    ];
}

#[derive(Debug)]
pub enum PlayerAction {
    Move(Direction),
    DropBomb,
}

#[derive(Debug)]
pub enum PlayerIntention {
    MoveToSafety,
    PickUpItem,
    DestroyBlocks,
    KillPlayers,
    PlaceBombNearPlayers,
    RandomMove,
    HuntPlayers,
    Flee,
}

#[derive(Clone, Copy)]
pub enum BotDifficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Power {
    WallHack,
    BombPush,
    Immortal,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::enum_variant_names)]
pub enum Upgrade {
    BombsUp,
    RangeUp,
    LivesUp,
}

pub enum CooldownState {
    Ready,
    CoolingDown(Timer),
}

pub struct Cooldown {
    state: CooldownState,
    duration: Duration,
}

impl Cooldown {
    pub fn from_seconds(duration: f32) -> Self {
        Cooldown {
            state: CooldownState::Ready,
            duration: Duration::from_secs_f32(duration),
        }
    }

    pub fn ready(&self) -> bool {
        matches!(self.state, CooldownState::Ready)
    }

    pub fn trigger(&mut self) {
        self.state =
            CooldownState::CoolingDown(Timer::from_seconds(self.duration.as_secs_f32(), false));
    }

    pub fn tick(&mut self, delta: Duration) {
        match self.state {
            CooldownState::Ready => (),
            CooldownState::CoolingDown(ref mut timer) => {
                timer.tick(delta);
                if timer.finished() {
                    self.state = CooldownState::Ready;
                }
            }
        };
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cooldown() {
        let mut cooldown = Cooldown::from_seconds(0.5);
        assert!(cooldown.ready());
        assert_eq!(cooldown.duration(), Duration::from_millis(500));

        cooldown.trigger();
        assert!(!cooldown.ready());

        cooldown.tick(Duration::from_secs_f32(0.3));
        assert!(!cooldown.ready());

        cooldown.tick(Duration::from_secs_f32(0.2));
        assert!(cooldown.ready());

        cooldown.trigger();
        assert!(!cooldown.ready());
    }
}
