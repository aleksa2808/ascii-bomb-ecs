use std::collections::HashMap;

use bevy::{core::Timer, prelude::*};

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

pub struct LeaderboardMaterials {
    pub trophy: Handle<ColorMaterial>,
}

impl FromWorld for LeaderboardMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut leaderboard_materials = None;
        world.resource_scope(|world, mut materials: Mut<Assets<ColorMaterial>>| {
            let asset_server = world.get_resource::<AssetServer>().unwrap();

            let trophy_texture = asset_server.load("sprites/trophy.png");

            leaderboard_materials = Some(LeaderboardMaterials {
                trophy: materials.add(trophy_texture.clone().into()),
            });

            if let Some(mut assets_loading) = world.get_resource_mut::<AssetsLoading>() {
                assets_loading.0.push(trophy_texture.clone_untyped());
            }
        });

        leaderboard_materials.expect("Leaderboard textures could not be loaded")
    }
}
