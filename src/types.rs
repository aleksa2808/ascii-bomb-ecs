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

    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Right => Direction::Left,
            Direction::Left => Direction::Right,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }
}

pub enum PlayerAction {
    Move(Direction),
    DropBomb,
}

#[derive(Clone, Copy)]
pub enum PenguinControlType {
    Human(usize),
    Bot,
}

// TODO: remove
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum BotDifficulty {
    Easy,
    Medium,
    Hard,
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

#[derive(Clone, Copy)]
pub struct RGBColor(pub u8, pub u8, pub u8);

impl From<RGBColor> for Color {
    fn from(color: RGBColor) -> Self {
        Self::rgb_u8(color.0, color.1, color.2)
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
