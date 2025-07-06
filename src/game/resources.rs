use bevy::{ecs as bevy_ecs, prelude::*};

use crate::{
    audio::{SoundHandles, SoundID},
    common::constants::COLORS,
    loading::resources::LoadingAssetHandles,
    AppState,
};

use super::{
    components::{Penguin, Position},
    types::{Cooldown, Direction},
};

#[derive(Default)]
pub struct MapTextures {
    pub empty: Handle<Image>,
    pub wall: Handle<Image>,
    pub destructible_wall: Handle<Image>,
    pub burning_wall: Handle<Image>,
}

#[derive(Resource)]
pub struct GameTextures {
    // players + effects
    penguin_variants: Vec<Handle<Image>>,
    pub immortal_penguin: Handle<Image>,
    pub crook: Handle<Image>,
    pub immortal_crook: Handle<Image>,
    pub hatter: Handle<Image>,
    pub immortal_hatter: Handle<Image>,
    pub bat: Handle<Image>,
    pub immortal_bat: Handle<Image>,
    // bomb + fire
    pub bomb: Handle<Image>,
    pub fire: Handle<Image>,
    // map textures
    map_textures: Vec<MapTextures>,
    map_textures_index: usize,
    // exit
    pub exit: Handle<Image>,
    // items
    pub bombs_up: Handle<Image>,
    pub range_up: Handle<Image>,
    pub lives_up: Handle<Image>,
    pub wall_hack: Handle<Image>,
    pub bomb_push: Handle<Image>,
    pub immortal: Handle<Image>,
    pub burning_item: Handle<Image>,
}

impl GameTextures {
    pub fn set_map_textures(&mut self, world_id: WorldID) {
        self.map_textures_index = world_id.0 - 1;
    }

    pub fn get_map_textures(&self) -> &MapTextures {
        &self.map_textures[self.map_textures_index]
    }

    pub fn get_penguin_texture(&self, penguin: Penguin) -> &Handle<Image> {
        self.penguin_variants.iter().cycle().nth(penguin.0).unwrap()
    }
}

impl FromWorld for GameTextures {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        let map_textures: Vec<MapTextures> = (1..=3)
            .map(|world_id| MapTextures {
                empty: asset_server.load(format!("sprites/world/{}/empty.png", world_id)),
                wall: asset_server.load(format!("sprites/world/{}/wall.png", world_id)),
                destructible_wall: asset_server
                    .load(format!("sprites/world/{}/destructible_wall.png", world_id)),
                burning_wall: asset_server
                    .load(format!("sprites/world/{}/burning_wall.png", world_id)),
            })
            .collect();

        let penguin_variants: Vec<Handle<Image>> = (0..=14)
            .map(|i| asset_server.load(format!("sprites/penguins/{}.png", i)))
            .collect();

        let immortal_penguin_texture = asset_server.load("sprites/immortal_penguin.png");
        let crook_texture = asset_server.load("sprites/crook.png");
        let immortal_crook_texture = asset_server.load("sprites/immortal_crook.png");
        let hatter_texture = asset_server.load("sprites/hatter.png");
        let immortal_hatter_texture = asset_server.load("sprites/immortal_hatter.png");
        let bat_texture = asset_server.load("sprites/bat.png");
        let immortal_bat_texture = asset_server.load("sprites/immortal_bat.png");
        let bomb_texture = asset_server.load("sprites/bomb.png");
        let fire_texture = asset_server.load("sprites/fire.png");
        let exit_texture = asset_server.load("sprites/exit.png");
        let bombs_up_texture = asset_server.load("sprites/bombs_up.png");
        let range_up_texture = asset_server.load("sprites/range_up.png");
        let lives_up_texture = asset_server.load("sprites/lives_up.png");
        let wall_hack_texture = asset_server.load("sprites/wall_hack.png");
        let bomb_push_texture = asset_server.load("sprites/bomb_push.png");
        let immortal_texture = asset_server.load("sprites/immortal.png");
        let burning_item_texture = asset_server.load("sprites/burning_item.png");

        let game_textures = GameTextures {
            // players + effects
            penguin_variants: penguin_variants.to_vec(),
            immortal_penguin: immortal_penguin_texture.clone(),
            crook: crook_texture.clone(),
            immortal_crook: immortal_crook_texture.clone(),
            hatter: hatter_texture.clone(),
            immortal_hatter: immortal_hatter_texture.clone(),
            bat: bat_texture.clone(),
            immortal_bat: immortal_bat_texture.clone(),
            // bomb + fire
            bomb: bomb_texture.clone(),
            fire: fire_texture.clone(),
            // map textures
            map_textures: map_textures
                .iter()
                .map(|mt| MapTextures {
                    empty: mt.empty.clone(),
                    wall: mt.wall.clone(),
                    destructible_wall: mt.destructible_wall.clone(),
                    burning_wall: mt.burning_wall.clone(),
                })
                .collect(),
            map_textures_index: 0, // defaults to world 1
            // exit
            exit: exit_texture.clone(),
            // items
            bombs_up: bombs_up_texture.clone(),
            range_up: range_up_texture.clone(),
            lives_up: lives_up_texture.clone(),
            wall_hack: wall_hack_texture.clone(),
            bomb_push: bomb_push_texture.clone(),
            immortal: immortal_texture.clone(),
            burning_item: burning_item_texture.clone(),
        };

        // register the textures in cLoadingAssetHandles
        if let Some(mut loading_asset_handles) = world.get_resource_mut::<LoadingAssetHandles>() {
            loading_asset_handles
                .0
                .extend(map_textures.into_iter().flat_map(|mt| {
                    vec![
                        mt.empty.untyped(),
                        mt.wall.untyped(),
                        mt.destructible_wall.untyped(),
                        mt.burning_wall.untyped(),
                    ]
                }));

            loading_asset_handles
                .0
                .extend(penguin_variants.into_iter().map(|pt| pt.untyped()));

            loading_asset_handles.0.append(&mut vec![
                immortal_penguin_texture.untyped(),
                crook_texture.untyped(),
                immortal_crook_texture.untyped(),
                hatter_texture.untyped(),
                immortal_hatter_texture.untyped(),
                bat_texture.untyped(),
                immortal_bat_texture.untyped(),
                bomb_texture.untyped(),
                fire_texture.untyped(),
                exit_texture.untyped(),
                bombs_up_texture.untyped(),
                range_up_texture.untyped(),
                lives_up_texture.untyped(),
                wall_hack_texture.untyped(),
                bomb_push_texture.untyped(),
                immortal_texture.untyped(),
                burning_item_texture.untyped(),
            ]);
        }

        game_textures
    }
}

#[derive(Resource)]
pub struct Sounds {
    pub boom: SoundID,
    pub pause: SoundID,
}

impl FromWorld for Sounds {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        let boom_handle = asset_server.load("sounds/boom.wav");
        let pause_handle = asset_server.load("sounds/pause.wav");

        if let Some(mut loading_asset_handles) = world.get_resource_mut::<LoadingAssetHandles>() {
            loading_asset_handles.0.append(&mut vec![
                boom_handle.clone().untyped(),
                pause_handle.clone().untyped(),
            ]);
        }

        let mut sound_handles = world.get_resource_mut::<SoundHandles>().unwrap();
        Self {
            boom: sound_handles.add_handle(boom_handle),
            pause: sound_handles.add_handle(pause_handle),
        }
    }
}

// HUD display
#[derive(Resource)]
pub struct HUDColors {
    background_colors: Vec<Color>,
    pub black_color: Color,
    pub portrait_background_color: Color,
    pub portrait_border_color: Color,
}

impl HUDColors {
    pub fn get_background_color(&self, world_id: WorldID) -> Color {
        self.background_colors[world_id.0 - 1]
    }
}

impl Default for HUDColors {
    fn default() -> Self {
        Self {
            background_colors: vec![
                Color::into(COLORS[2].into()),
                Color::into(COLORS[11].into()),
                Color::into(COLORS[3].into()),
            ],
            black_color: COLORS[0].into(),
            portrait_background_color: COLORS[3].into(),
            portrait_border_color: COLORS[8].into(),
        }
    }
}

#[derive(Clone, Copy, Resource)]
pub struct MapSize {
    pub rows: usize,
    pub columns: usize,
}

// pretty similar to the ECS WorldId type, maybe rename?
#[derive(Clone, Copy, Resource)]
pub struct WorldID(pub usize);

#[derive(Resource)]
pub struct GameTimer(pub Timer);

#[derive(Resource)]
pub struct GameScore(pub usize);

#[derive(Resource)]
pub struct GameContext {
    pub pausable: bool,
    pub reduced_loot: bool,
    pub game_mode_manager_state: AppState,
}

#[derive(Resource)]
pub struct PauseContext {
    pub in_game_state: AppState,
}

#[derive(Resource)]
pub struct ExitPosition(pub Position);

pub struct ActiveWallOfDeath {
    pub position: Position,
    pub direction: Direction,
    pub cooldown: Cooldown,
}

#[derive(Resource)]
pub enum WallOfDeath {
    Dormant(Timer),
    Active(ActiveWallOfDeath),
    Done,
}
