use std::collections::HashMap;

use bevy::{core::Timer, prelude::Entity};

use crate::game::{
    components::{Penguin, Position},
    types::{BotDifficulty, Cooldown, Direction, PenguinControlType},
};

pub struct BattleModeConfiguration {
    pub amount_of_players: usize,
    pub amount_of_bots: usize,
    pub winning_score: usize,
    pub difficulty: BotDifficulty,
}

pub enum BattleModeState {
    RoundSetup,
    MapTransition,
    InGame,
    LeaderboardDisplay,
}

pub struct Leaderboard {
    pub scores: HashMap<Penguin, usize>,
    pub winning_score: usize,
}

#[derive(Clone, Copy)]
pub enum RoundOutcome {
    Winner(Penguin),
    Tie,
}

pub struct BattleModeContext {
    pub state: BattleModeState,
    pub players: Vec<(Penguin, PenguinControlType)>,
    pub leaderboard: Leaderboard,
    pub round_outcome: Option<RoundOutcome>,
    // cache of the map block fill ratio
    pub percent_of_passable_positions_to_fill: f32,
}

pub struct ActiveWallOfDeath {
    pub position: Position,
    pub direction: Direction,
    pub cooldown: Cooldown,
}

pub enum WallOfDeath {
    Dormant(Timer),
    Active(ActiveWallOfDeath),
    Done,
}

// round start freeze
pub struct FreezeTimer(pub Timer);

// leaderboard display
pub struct LeaderboardDisplayContext {
    pub leaderboard_display_box: Entity,
    pub timer: Timer,
}
