use bevy::prelude::*;
use bevy_kira_audio::AudioSource;

use crate::{common::constants::COLORS, loading::resources::AssetsLoading};

use super::{
    components::{Penguin, Position},
    types::{Cooldown, Direction},
};

struct MapTextures {
    pub empty: Handle<Texture>,
    pub wall: Handle<Texture>,
    pub destructible_wall: Handle<Texture>,
    pub burning_wall: Handle<Texture>,
}

#[derive(Default)]
pub struct MapMaterials {
    pub empty: Handle<ColorMaterial>,
    pub wall: Handle<ColorMaterial>,
    pub destructible_wall: Handle<ColorMaterial>,
    pub burning_wall: Handle<ColorMaterial>,
}

pub struct GameMaterials {
    // players + effects
    penguin_variants: Vec<Handle<ColorMaterial>>,
    pub immortal_penguin: Handle<ColorMaterial>,
    pub crook: Handle<ColorMaterial>,
    pub immortal_crook: Handle<ColorMaterial>,
    pub hatter: Handle<ColorMaterial>,
    pub immortal_hatter: Handle<ColorMaterial>,
    pub bat: Handle<ColorMaterial>,
    pub immortal_bat: Handle<ColorMaterial>,
    // bomb + fire
    pub bomb: Handle<ColorMaterial>,
    pub fire: Handle<ColorMaterial>,
    // map materials
    map_materials: Vec<MapMaterials>,
    map_materials_index: usize,
    // exit
    pub exit: Handle<ColorMaterial>,
    // items
    pub bombs_up: Handle<ColorMaterial>,
    pub range_up: Handle<ColorMaterial>,
    pub lives_up: Handle<ColorMaterial>,
    pub wall_hack: Handle<ColorMaterial>,
    pub bomb_push: Handle<ColorMaterial>,
    pub immortal: Handle<ColorMaterial>,
    pub burning_item: Handle<ColorMaterial>,
}

impl GameMaterials {
    pub fn set_map_materials(&mut self, world_id: WorldID) {
        self.map_materials_index = world_id.0 - 1;
    }

    pub fn get_map_materials(&self) -> &MapMaterials {
        &self.map_materials[self.map_materials_index]
    }

    pub fn get_penguin_material(&self, penguin: Penguin) -> &Handle<ColorMaterial> {
        self.penguin_variants.iter().cycle().nth(penguin.0).unwrap()
    }
}

impl FromWorld for GameMaterials {
    fn from_world(world: &mut World) -> Self {
        // TODO: there's gotta be an easier (and less error prone) way to load textures + register them in AssetsLoading
        let mut game_materials = None;
        world.resource_scope(|world, mut materials: Mut<Assets<ColorMaterial>>| {
            let asset_server = world.get_resource::<AssetServer>().unwrap();

            // load the textures
            let map_textures: Vec<MapTextures> = (1..=3)
                .map(|world_id| MapTextures {
                    empty: asset_server
                        .load(format!("sprites/world/{}/empty.png", world_id).as_str()),
                    wall: asset_server
                        .load(format!("sprites/world/{}/wall.png", world_id).as_str()),
                    destructible_wall: asset_server
                        .load(format!("sprites/world/{}/destructible_wall.png", world_id).as_str()),
                    burning_wall: asset_server
                        .load(format!("sprites/world/{}/burning_wall.png", world_id).as_str()),
                })
                .collect();

            let penguin_textures: Vec<Handle<Texture>> = (0..=14)
                .map(|i| asset_server.load(format!("sprites/penguins/{}.png", i).as_str()))
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

            // create materials out of the loaded textures
            let map_materials = map_textures
                .iter()
                .map(|mt| MapMaterials {
                    empty: materials.add(mt.empty.clone().into()),
                    wall: materials.add(mt.wall.clone().into()),
                    destructible_wall: materials.add(mt.destructible_wall.clone().into()),
                    burning_wall: materials.add(mt.burning_wall.clone().into()),
                })
                .collect();

            let penguin_variants = penguin_textures
                .iter()
                .map(|pt| materials.add((*pt).clone().into()))
                .collect();

            game_materials = Some(GameMaterials {
                // players + effects
                penguin_variants,
                immortal_penguin: materials.add(immortal_penguin_texture.clone().into()),
                crook: materials.add(crook_texture.clone().into()),
                immortal_crook: materials.add(immortal_crook_texture.clone().into()),
                hatter: materials.add(hatter_texture.clone().into()),
                immortal_hatter: materials.add(immortal_hatter_texture.clone().into()),
                bat: materials.add(bat_texture.clone().into()),
                immortal_bat: materials.add(immortal_bat_texture.clone().into()),
                // bomb + fire
                bomb: materials.add(bomb_texture.clone().into()),
                fire: materials.add(fire_texture.clone().into()),
                // map materials
                map_materials,
                map_materials_index: 0, // defaults to world 1
                // exit
                exit: materials.add(exit_texture.clone().into()),
                // items
                bombs_up: materials.add(bombs_up_texture.clone().into()),
                range_up: materials.add(range_up_texture.clone().into()),
                lives_up: materials.add(lives_up_texture.clone().into()),
                wall_hack: materials.add(wall_hack_texture.clone().into()),
                bomb_push: materials.add(bomb_push_texture.clone().into()),
                immortal: materials.add(immortal_texture.clone().into()),
                burning_item: materials.add(burning_item_texture.clone().into()),
            });

            // register the textures in AssetsLoading
            if let Some(mut assets_loading) = world.get_resource_mut::<AssetsLoading>() {
                assets_loading.0.extend(map_textures.iter().flat_map(|mt| {
                    vec![
                        mt.empty.clone_untyped(),
                        mt.wall.clone_untyped(),
                        mt.destructible_wall.clone_untyped(),
                        mt.burning_wall.clone_untyped(),
                    ]
                }));

                assets_loading
                    .0
                    .extend(penguin_textures.iter().map(|pt| pt.clone_untyped()));

                assets_loading.0.append(&mut vec![
                    immortal_penguin_texture.clone_untyped(),
                    crook_texture.clone_untyped(),
                    immortal_crook_texture.clone_untyped(),
                    hatter_texture.clone_untyped(),
                    immortal_hatter_texture.clone_untyped(),
                    bat_texture.clone_untyped(),
                    immortal_bat_texture.clone_untyped(),
                    bomb_texture.clone_untyped(),
                    fire_texture.clone_untyped(),
                    exit_texture.clone_untyped(),
                    bombs_up_texture.clone_untyped(),
                    range_up_texture.clone_untyped(),
                    lives_up_texture.clone_untyped(),
                    wall_hack_texture.clone_untyped(),
                    bomb_push_texture.clone_untyped(),
                    immortal_texture.clone_untyped(),
                    burning_item_texture.clone_untyped(),
                ]);
            }
        });

        game_materials.expect("Game textures could not be loaded")
    }
}

pub struct Sounds {
    pub boom: Handle<AudioSource>,
    pub pause: Handle<AudioSource>,
}

impl FromWorld for Sounds {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        let sounds = Sounds {
            boom: asset_server.load("sounds/boom.wav"),
            pause: asset_server.load("sounds/pause.wav"),
        };

        if let Some(mut assets_loading) = world.get_resource_mut::<AssetsLoading>() {
            assets_loading.0.append(&mut vec![
                sounds.boom.clone_untyped(),
                sounds.pause.clone_untyped(),
            ]);
        }

        sounds
    }
}

// HUD display
pub struct HUDMaterials {
    background_materials: Vec<Handle<ColorMaterial>>,
    pub black: Handle<ColorMaterial>,
    pub portrait_background_color: Handle<ColorMaterial>,
    pub portrait_border_color: Handle<ColorMaterial>,
}

impl HUDMaterials {
    pub fn get_background_material(&self, world_id: WorldID) -> &Handle<ColorMaterial> {
        &self.background_materials[world_id.0 - 1]
    }
}

impl FromWorld for HUDMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();

        let background_materials = vec![
            materials.add(Color::into(COLORS[2].into())),
            materials.add(Color::into(COLORS[11].into())),
            materials.add(Color::into(COLORS[3].into())),
        ];

        let black: Color = COLORS[0].into();
        let portrait_background_color: Color = COLORS[3].into();
        let portrait_border_color: Color = COLORS[8].into();

        HUDMaterials {
            background_materials,
            black: materials.add(black.into()),
            portrait_background_color: materials.add(portrait_background_color.into()),
            portrait_border_color: materials.add(portrait_border_color.into()),
        }
    }
}

#[derive(Clone, Copy)]
pub struct MapSize {
    pub rows: usize,
    pub columns: usize,
}

// pretty similar to the ECS WorldId type, maybe rename?
#[derive(Clone, Copy)]
pub struct WorldID(pub usize);

pub struct GameTimer(pub Timer);

pub struct GameScore(pub usize);

pub struct GameContext {
    pub pausable: bool,
    pub reduced_loot: bool,
}

pub struct ExitPosition(pub Position);

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
