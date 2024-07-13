use bevy::{ecs as bevy_ecs, prelude::*, utils::HashMap};

use crate::{
    game::{components::Penguin, types::BotDifficulty},
    loading::resources::LoadingAssetHandles,
};

use super::types::PenguinControlType;

#[derive(Resource)]
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

#[derive(Resource)]
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
#[derive(Resource)]
pub struct FreezeTimer(pub Timer);

// leaderboard display
#[derive(Resource)]
pub struct LeaderboardDisplayContext {
    pub leaderboard_display_box: Entity,
    pub timer: Timer,
}

#[derive(Resource)]
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

        if let Some(mut loading_asset_handles) = world.get_resource_mut::<LoadingAssetHandles>() {
            loading_asset_handles.0.push(trophy_texture.untyped());
        }

        leaderboard_textures
    }
}
