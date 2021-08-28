use std::{collections::HashMap, time::Duration};

use bevy::prelude::*;

use crate::{
    components::{Penguin, Position},
    constants::COLORS,
    types::{Cooldown, Direction},
};

pub struct BaseColorMaterials {
    pub none: Handle<ColorMaterial>,
    pub colors: Vec<Handle<ColorMaterial>>,
}

impl FromWorld for BaseColorMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        Self {
            none: materials.add(Color::NONE.into()),
            colors: COLORS
                .iter()
                .copied()
                .map(|color| {
                    let color: Color = color.into();
                    materials.add(color.into())
                })
                .collect(),
        }
    }
}

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

pub struct Fonts {
    pub mono: Handle<Font>,
}

impl FromWorld for Fonts {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        Fonts {
            mono: asset_server.load("fonts/UbuntuMono-R.ttf"),
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

// splash screen
pub enum SplashScreenTextState {
    Moving(Timer),
    Holding(Timer),
}

pub struct SplashScreenContext {
    pub left_text: Entity,
    pub right_text: Entity,
    pub left_position: usize,
    pub right_position: usize,
    pub text_state: SplashScreenTextState,
}

// menu
pub struct MenuMaterials {
    pub background_color: Color,
    pub modal_background_color: Color,
    pub modal_foreground_color: Color,
    pub background: Handle<ColorMaterial>,
    pub modal_background: Handle<ColorMaterial>,
    pub modal_foreground: Handle<ColorMaterial>,
}

impl FromWorld for MenuMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        let background_color: Color = COLORS[0].into();
        let modal_background_color: Color = COLORS[8].into();
        let modal_foreground_color: Color = COLORS[1].into();
        MenuMaterials {
            background_color,
            modal_background_color,
            modal_foreground_color,
            background: materials.add(background_color.into()),
            modal_background: materials.add(modal_background_color.into()),
            modal_foreground: materials.add(modal_foreground_color.into()),
        }
    }
}

#[derive(Clone, Copy)]
pub enum MenuAction {
    SwitchMenu(usize),
    LaunchStoryMode,
    LaunchBattleMode,
    Back,
    Exit,
    Disabled, // TODO: remove
}

#[derive(Clone)]
pub struct SelectableItems {
    items: Vec<(&'static str, MenuAction)>,
    cursor_position: usize,
}

impl SelectableItems {
    pub fn get_option_names(&self) -> Vec<&'static str> {
        self.items
            .iter()
            .map(|(s, _)| *s)
            .collect::<Vec<&'static str>>()
    }

    pub fn get_action(&self) -> MenuAction {
        self.items[self.cursor_position].1
    }

    pub fn get_cursor_position(&self) -> usize {
        self.cursor_position
    }

    pub fn move_cursor(&mut self, direction: Direction) {
        match direction {
            Direction::Up => {
                if self.cursor_position == 0 {
                    self.cursor_position = self.items.len() - 1;
                } else {
                    self.cursor_position -= 1;
                }
            }
            Direction::Down => {
                if self.cursor_position == self.items.len() - 1 {
                    self.cursor_position = 0;
                } else {
                    self.cursor_position += 1;
                }
            }
            _ => (),
        }
    }
}

#[derive(Clone)]
pub enum MenuType {
    SelectableItems(SelectableItems),
    StaticText(&'static str),
}

pub struct MenuState {
    menu_types: Vec<MenuType>,
    menu_stack: Vec<MenuType>,
}

impl Default for MenuState {
    fn default() -> Self {
        let initial_state = MenuType::SelectableItems(SelectableItems {
            items: vec![
                ("PLAY", MenuAction::SwitchMenu(1)),
                ("OPTIONS", MenuAction::Disabled),
                ("HELP", MenuAction::SwitchMenu(2)),
                ("HALL OF FAME", MenuAction::Disabled),
                ("EXIT", MenuAction::Exit),
            ],
            cursor_position: 0,
        });

        Self {
            menu_types: vec![
                initial_state.clone(),
                MenuType::SelectableItems(SelectableItems {
                    items: vec![
                        ("STORY MODE", MenuAction::LaunchStoryMode),
                        ("BATTLE MODE", MenuAction::LaunchBattleMode),
                    ],
                    cursor_position: 0,
                }),
                MenuType::SelectableItems(SelectableItems {
                    items: vec![
                        ("ABOUT", MenuAction::SwitchMenu(3)),
                        ("CONTROLS", MenuAction::SwitchMenu(4)),
                        ("POWER-UPS", MenuAction::SwitchMenu(5)),
                    ],
                    cursor_position: 0,
                }),
                MenuType::StaticText(
                    r#"
You are a penguin. With a top hat.
You also have bombs.
You also have enemies.
Some of them also have bombs.
Bombs can kill enemies.
So, try to kill your enemies.

Made by
              Aleksa Pavlovic
              Nikola Vaic
              Dusan Mrvaljevic
"#,
                ),
                MenuType::StaticText(
                    r#"
Arrow Keys    - P1 movement
Space Bar     - P1 bomb set
WASD Keys     - P2 movement
G Key         - P2 bomb set
Enter Key     - Pause
ESC Key       - Back




F Key - ???
"#,
                ),
                MenuType::StaticText(
                    r#"
LIFE UP       - H+
RANGE UP      - R+
PUSHING       - Boot
WALL CLIMB    - Ladders
INVINCIBILITY - Top Hat
"#,
                ),
            ],
            menu_stack: vec![initial_state],
        }
    }
}

impl MenuState {
    pub fn get_current_menu(&self) -> &MenuType {
        self.menu_stack.last().unwrap()
    }

    pub fn get_current_menu_mut(&mut self) -> &mut MenuType {
        self.menu_stack.last_mut().unwrap()
    }

    pub fn get_action(&self) -> MenuAction {
        match self.get_current_menu() {
            MenuType::SelectableItems(selectable_items) => selectable_items.get_action(),
            MenuType::StaticText(_) => MenuAction::Back,
        }
    }

    pub fn switch_menu(&mut self, menu_id: usize) {
        self.menu_stack.push(self.menu_types[menu_id].clone());
    }

    pub fn back(&mut self) -> Result<(), ()> {
        if self.menu_stack.len() > 1 {
            self.menu_stack.pop();
            Ok(())
        } else {
            Err(())
        }
    }
}

pub enum MenuBackgroundEntityValues {
    Text(String, String),
    Color(Color, Color),
}

pub struct MenuBackgroundEntityChangeParameters {
    pub modulo: usize,
    pub threshold: usize,
    pub values: MenuBackgroundEntityValues,
}

pub struct MenuBackgroundAnimationContext {
    pub entity_change_parameters: HashMap<Entity, MenuBackgroundEntityChangeParameters>,
    pub timer: Timer,
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

// boss speech
pub struct BossSpeechScript {
    script: Vec<(Penguin, &'static str)>,
    line_index: usize,
    line_character: usize,
    line_timer: Timer,
}

impl BossSpeechScript {
    pub fn new(script: Vec<(Penguin, &'static str)>) -> Self {
        Self {
            script,
            line_index: 0,
            line_character: 0,
            line_timer: Timer::from_seconds(0.02, true),
        }
    }

    pub fn tick(&mut self, delta: Duration) {
        self.line_timer.tick(delta);
        if self.line_in_progress() {
            self.line_character = (self.line_character + self.line_timer.times_finished() as usize)
                .min(self.get_current_line().len() - 1);
            if !self.line_in_progress() {
                self.line_timer.pause();
            }
        }
    }

    pub fn get_current_speaker(&self) -> Penguin {
        self.script[self.line_index].0
    }

    fn get_current_line(&self) -> &'static str {
        self.script[self.line_index].1
    }

    pub fn get_current_line_state(&self) -> &'static str {
        &self.get_current_line()[0..=self.line_character]
    }

    pub fn line_in_progress(&self) -> bool {
        self.line_character < self.get_current_line().len() - 1
    }

    pub fn complete_current_line(&mut self) {
        if self.line_in_progress() {
            self.line_character = self.get_current_line().len() - 1;
            self.line_timer.pause();
        }
    }

    pub fn advance_script(&mut self) -> Result<(), ()> {
        if self.line_index < self.script.len() - 1 {
            self.line_index += 1;
            self.line_character = 0;
            self.line_timer.reset();
            self.line_timer.unpause();
            Ok(())
        } else {
            Err(())
        }
    }
}

pub struct BossSpeechBoxEntities {
    pub speech_box: Entity,
    pub speaker_portrait: Entity,
    pub speech_text: Entity,
}

// battle mode
pub struct Leaderboard {
    pub scores: HashMap<Penguin, usize>,
    pub last_round_winner: Option<Penguin>,
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

// leaderboard display
pub struct LeaderboardDisplay {
    pub leaderboard_display_box: Entity,
    pub timer: Timer,
}
