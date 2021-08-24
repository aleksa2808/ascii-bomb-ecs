use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    components::Position,
    constants::COLORS,
    types::{Cooldown, Direction},
};

#[derive(Default)]
pub struct MapTextures {
    pub empty: Handle<ColorMaterial>,
    pub wall: Handle<ColorMaterial>,
    pub destructible_wall: Handle<ColorMaterial>,
    pub burning_wall: Handle<ColorMaterial>,
}

pub struct Textures {
    // players + effects
    pub penguin_variants: Vec<Handle<ColorMaterial>>,
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
    pub map_textures: Vec<MapTextures>,
    pub map_textures_index: usize,
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

impl Textures {
    pub fn set_map_textures(&mut self, world: usize) {
        self.map_textures_index = world - 1;
    }

    pub fn get_map_textures(&self) -> &MapTextures {
        &self.map_textures[self.map_textures_index]
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
            });
        });

        textures.expect("Textures could not be loaded")
    }
}

pub struct Fonts {
    pub mono: Handle<Font>,
    pub bold: Handle<Font>,
}

impl FromWorld for Fonts {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        Fonts {
            mono: asset_server.load("fonts/UbuntuMono-R.ttf"),
            bold: asset_server.load("fonts/FiraSans-Bold.ttf"),
        }
    }
}

// menu
pub struct MenuMaterials {
    pub modal_background_color: Color,
    pub modal_foreground_color: Color,
    pub modal_backround: Handle<ColorMaterial>,
    pub modal_foreground: Handle<ColorMaterial>,
}

impl FromWorld for MenuMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        let modal_background_color: Color = COLORS[8].into();
        let modal_foreground_color: Color = COLORS[1].into();
        MenuMaterials {
            modal_background_color,
            modal_foreground_color,
            modal_backround: materials.add(modal_background_color.into()),
            modal_foreground: materials.add(modal_foreground_color.into()),
        }
    }
}

#[derive(Clone, Copy)]
pub enum MenuAction {
    SwitchMenu(usize),
    LaunchStoryMode,
    LaunchBattleMode,
    Exit,
    Disabled, // TODO: remove
}

pub struct MenuState {
    options: Vec<Vec<(&'static str, MenuAction)>>,
    menu_id_stack: Vec<usize>,
    cursor_position: usize,
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            options: vec![
                vec![
                    ("PLAY", MenuAction::SwitchMenu(1)),
                    ("OPTIONS", MenuAction::Disabled),
                    ("HELP", MenuAction::SwitchMenu(2)),
                    ("HALL OF FAME", MenuAction::Disabled),
                    ("EXIT", MenuAction::Exit),
                ],
                vec![
                    ("STORY MODE", MenuAction::LaunchStoryMode),
                    ("BATTLE MODE", MenuAction::LaunchBattleMode),
                ],
                vec![
                    ("ABOUT", MenuAction::Disabled),
                    ("CONTROLS", MenuAction::Disabled),
                    ("POWER-UPS", MenuAction::Disabled),
                ],
            ],
            menu_id_stack: vec![0],
            cursor_position: 0,
        }
    }
}

impl MenuState {
    fn get_options(&self) -> &Vec<(&'static str, MenuAction)> {
        &self.options[*self.menu_id_stack.last().unwrap()]
    }

    pub fn get_option_names(&self) -> Vec<&'static str> {
        self.options[*self.menu_id_stack.last().unwrap()]
            .iter()
            .map(|(s, _)| *s)
            .collect::<Vec<&'static str>>()
    }

    pub fn get_action(&self) -> MenuAction {
        self.options[*self.menu_id_stack.last().unwrap()][self.cursor_position].1
    }

    pub fn get_cursor_position(&self) -> usize {
        self.cursor_position
    }

    pub fn move_cursor(&mut self, direction: Direction) {
        match direction {
            Direction::Up => {
                if self.cursor_position == 0 {
                    self.cursor_position = self.get_options().len() - 1;
                } else {
                    self.cursor_position -= 1;
                }
            }
            Direction::Down => {
                if self.cursor_position == self.get_options().len() - 1 {
                    self.cursor_position = 0;
                } else {
                    self.cursor_position += 1;
                }
            }
            _ => (),
        }
    }

    pub fn switch_menu(&mut self, menu_id: usize) {
        self.menu_id_stack.push(menu_id);
        self.cursor_position = 0;
    }

    pub fn back(&mut self) -> Result<(), ()> {
        if self.menu_id_stack.len() > 1 {
            self.menu_id_stack.pop();
            self.cursor_position = 0;
            Ok(())
        } else {
            Err(())
        }
    }
}

// pretty similar to the ECS WorldId type, maybe rename?
#[derive(Clone, Copy)]
pub struct WorldID(pub usize);

pub struct GameTimer(pub Timer);

// story mode
#[derive(Clone, Copy)]
pub enum Level {
    Regular(usize),
    BossRoom,
}

pub struct GameScore(pub usize);

pub struct ExitPosition(pub Position);

// battle mode
pub struct Leaderboard {
    pub scores: HashMap<usize, usize>,
    pub winning_score: usize,
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
