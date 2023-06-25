use bevy::{prelude::*, utils::HashMap};

use crate::{
    game::{components::Penguin, types::BotDifficulty},
    loading::resources::AssetsLoading,
};

use super::types::PenguinControlType;

pub struct BattleModeConfiguration {
    pub amount_of_players: usize,
    pub amount_of_bots: usize,
    pub winning_score: usize,
    pub bot_difficulty: BotDifficulty,
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
    pub bot_difficulty: BotDifficulty,
    pub leaderboard: Leaderboard,
    pub round_outcome: Option<RoundOutcome>,
    // cache of the map block fill ratio
    pub percent_of_passable_positions_to_fill: f32,
}

// round start freeze
pub struct FreezeTimer(pub Timer);

// leaderboard display
pub struct LeaderboardDisplayContext {
    pub leaderboard_display_box: Entity,
    pub timer: Timer,
}

pub struct LeaderboardTextures {
    pub trophy: Handle<Image>,
}

impl FromWorld for LeaderboardTextures {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        let trophy_texture = asset_server.load("sprites/trophy.png");

        let leaderboard_textures = LeaderboardTextures {
            trophy: trophy_texture.clone(),
        };

        if let Some(mut assets_loading) = world.get_resource_mut::<AssetsLoading>() {
            assets_loading.0.push(trophy_texture.clone_untyped());
        }

        leaderboard_textures
    }
}
