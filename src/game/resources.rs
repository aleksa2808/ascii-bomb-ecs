use bevy::prelude::*;
use bevy_kira_audio::AudioSource;

use crate::common::constants::COLORS;

use super::components::{Penguin, Position};

#[derive(Default)]
pub struct MapTextures {
    pub empty: Handle<ColorMaterial>,
    pub wall: Handle<ColorMaterial>,
    pub destructible_wall: Handle<ColorMaterial>,
    pub burning_wall: Handle<ColorMaterial>,
}

pub struct Textures {
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
    // map textures
    map_textures: Vec<MapTextures>,
    map_textures_index: usize,
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
    // leaderboard
    pub trophy: Handle<ColorMaterial>,
}

impl Textures {
    pub fn set_map_textures(&mut self, world_id: WorldID) {
        self.map_textures_index = world_id.0 - 1;
    }

    pub fn get_map_textures(&self) -> &MapTextures {
        &self.map_textures[self.map_textures_index]
    }

    pub fn get_penguin_texture(&self, penguin: Penguin) -> &Handle<ColorMaterial> {
        self.penguin_variants.iter().cycle().nth(penguin.0).unwrap()
    }
}

impl FromWorld for Textures {
    fn from_world(world: &mut World) -> Self {
        let mut textures = None;

        world.resource_scope(|world, mut materials: Mut<Assets<ColorMaterial>>| {
            let asset_server = world.get_resource::<AssetServer>().unwrap();
            let map_textures = (1..=3)
                .map(|world_id| MapTextures {
                    empty: materials.add(
                        asset_server
                            .load(format!("sprites/world/{}/empty.png", world_id).as_str())
                            .into(),
                    ),
                    wall: materials.add(
                        asset_server
                            .load(format!("sprites/world/{}/wall.png", world_id).as_str())
                            .into(),
                    ),
                    destructible_wall: materials.add(
                        asset_server
                            .load(
                                format!("sprites/world/{}/destructible_wall.png", world_id)
                                    .as_str(),
                            )
                            .into(),
                    ),
                    burning_wall: materials.add(
                        asset_server
                            .load(format!("sprites/world/{}/burning_wall.png", world_id).as_str())
                            .into(),
                    ),
                })
                .collect();

            let penguin_variants = (0..=14)
                .map(|i| {
                    materials.add(
                        asset_server
                            .load(format!("sprites/penguins/{}.png", i).as_str())
                            .into(),
                    )
                })
                .collect();

            textures = Some(Textures {
                // players + effects
                penguin_variants,
                immortal_penguin: materials
                    .add(asset_server.load("sprites/immortal_penguin.png").into()),
                crook: materials.add(asset_server.load("sprites/crook.png").into()),
                immortal_crook: materials
                    .add(asset_server.load("sprites/immortal_crook.png").into()),
                hatter: materials.add(asset_server.load("sprites/hatter.png").into()),
                immortal_hatter: materials
                    .add(asset_server.load("sprites/immortal_hatter.png").into()),
                bat: materials.add(asset_server.load("sprites/bat.png").into()),
                immortal_bat: materials.add(asset_server.load("sprites/immortal_bat.png").into()),
                // bomb + fire
                bomb: materials.add(asset_server.load("sprites/bomb.png").into()),
                fire: materials.add(asset_server.load("sprites/fire.png").into()),
                // map textures
                map_textures,
                map_textures_index: 0, // defaults to world 1
                // exit
                exit: materials.add(asset_server.load("sprites/exit.png").into()),
                // items
                bombs_up: materials.add(asset_server.load("sprites/bombs_up.png").into()),
                range_up: materials.add(asset_server.load("sprites/range_up.png").into()),
                lives_up: materials.add(asset_server.load("sprites/lives_up.png").into()),
                wall_hack: materials.add(asset_server.load("sprites/wall_hack.png").into()),
                bomb_push: materials.add(asset_server.load("sprites/bomb_push.png").into()),
                immortal: materials.add(asset_server.load("sprites/immortal.png").into()),
                burning_item: materials.add(asset_server.load("sprites/burning_item.png").into()),
                trophy: materials.add(asset_server.load("sprites/trophy.png").into()),
            });
        });

        textures.expect("Textures could not be loaded")
    }
}

pub struct Sounds {
    pub boom: Handle<AudioSource>,
    pub pause: Handle<AudioSource>,
    pub what_is_f: Handle<AudioSource>,
}

impl FromWorld for Sounds {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        Sounds {
            boom: asset_server.load("sounds/boom.wav"),
            pause: asset_server.load("sounds/pause.wav"),
            what_is_f: asset_server.load("sounds/what_is_f.ogg"),
        }
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
