use std::collections::HashMap;

use bevy::prelude::*;
use bevy_kira_audio::AudioSource;

use crate::{constants::COLORS, resources::GameOption, types::BotDifficulty};

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

pub struct MainMenuSoundEffects {
    pub confirm: Handle<AudioSource>,
    pub select: Handle<AudioSource>,
}

impl FromWorld for MainMenuSoundEffects {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        MainMenuSoundEffects {
            confirm: asset_server.load("sounds/confirm.wav"),
            select: asset_server.load("sounds/select.wav"),
        }
    }
}

#[derive(Clone, Copy)]
pub enum MenuAction {
    SwitchMenu(usize),
    LaunchStoryMode,
    OpenBattleModeSubMenu,
    ToggleOption(GameOption),
    Back,
    Exit,
}

#[derive(Clone)]
pub struct SelectableItems {
    items: Vec<(&'static str, MenuAction)>,
    cursor_position: usize,
}

impl SelectableItems {
    pub fn get_item_names(&self) -> Vec<&'static str> {
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

    pub fn cycle_cursor_up(&mut self) {
        if self.cursor_position == self.items.len() - 1 {
            self.cursor_position = 0;
        } else {
            self.cursor_position += 1;
        }
    }

    pub fn cycle_cursor_down(&mut self) {
        if self.cursor_position == 0 {
            self.cursor_position = self.items.len() - 1;
        } else {
            self.cursor_position -= 1;
        }
    }
}

#[derive(Clone)]
pub struct ToggleableOptions {
    cursor_position: usize,
}

impl ToggleableOptions {
    pub fn get_options(&self) -> &[GameOption] {
        &GameOption::LIST
    }

    pub fn get_action(&self) -> MenuAction {
        MenuAction::ToggleOption(GameOption::LIST[self.cursor_position])
    }

    pub fn get_cursor_position(&self) -> usize {
        self.cursor_position
    }

    pub fn cycle_cursor_up(&mut self) {
        if self.cursor_position == GameOption::LIST.len() - 1 {
            self.cursor_position = 0;
        } else {
            self.cursor_position += 1;
        }
    }

    pub fn cycle_cursor_down(&mut self) {
        if self.cursor_position == 0 {
            self.cursor_position = GameOption::LIST.len() - 1;
        } else {
            self.cursor_position -= 1;
        }
    }
}

#[derive(Clone)]
pub enum MenuType {
    SelectableItems(SelectableItems),
    ToggleableOptions(ToggleableOptions),
    StaticText(&'static str),
    ControlsScreen(&'static str),
    HallOfFame,
}

pub enum BattleModeSubMenuStep {
    AmountOfPlayers,
    AmountOfBots,
    WinningScore,
    Difficulty,
}

pub struct ConfigurationOption<T> {
    values: Vec<T>,
    index: usize,
}

impl<T> ConfigurationOption<T> {
    pub fn new(values: Vec<T>, initial_value_index: usize) -> Result<Self, String> {
        // TODO: can this check be done at compile time?
        if initial_value_index < values.len() {
            Ok(Self {
                values,
                index: initial_value_index,
            })
        } else {
            Err(format!(
                "Index ({}) must not be higher than the values length ({}).",
                initial_value_index,
                values.len()
            ))
        }
    }

    pub fn value(&self) -> &T {
        &self.values[self.index]
    }

    pub fn increment(&mut self) {
        if self.index < self.values.len() - 1 {
            self.index += 1;
        }
    }

    pub fn decrement(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
    }
}

pub struct BattleModeSubMenuState {
    pub step: BattleModeSubMenuStep,
    pub amount_of_players: ConfigurationOption<usize>,
    pub amount_of_bots: ConfigurationOption<usize>,
    pub winning_score: ConfigurationOption<usize>,
    pub difficulty: ConfigurationOption<BotDifficulty>,
}

impl Default for BattleModeSubMenuState {
    fn default() -> Self {
        Self {
            step: BattleModeSubMenuStep::AmountOfPlayers,
            // TODO: expand this option
            amount_of_players: ConfigurationOption::new((1..=1).collect(), 0).unwrap(),
            amount_of_bots: ConfigurationOption::new((1..=7).collect(), 2).unwrap(),
            winning_score: ConfigurationOption::new((1..=5).collect(), 2).unwrap(),
            difficulty: ConfigurationOption::new(
                vec![
                    BotDifficulty::Easy,
                    BotDifficulty::Medium,
                    BotDifficulty::Hard,
                ],
                1,
            )
            .unwrap(),
        }
    }
}

pub struct MenuState {
    menu_types: Vec<MenuType>,
    menu_stack: Vec<MenuType>,
    pub battle_mode_sub_menu_state: Option<BattleModeSubMenuState>,
}

impl Default for MenuState {
    fn default() -> Self {
        let initial_state = MenuType::SelectableItems(SelectableItems {
            items: vec![
                ("PLAY", MenuAction::SwitchMenu(1)),
                ("OPTIONS", MenuAction::SwitchMenu(2)),
                ("HELP", MenuAction::SwitchMenu(3)),
                ("HALL OF FAME", MenuAction::SwitchMenu(4)),
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
                        ("BATTLE MODE", MenuAction::OpenBattleModeSubMenu),
                    ],
                    cursor_position: 0,
                }),
                MenuType::ToggleableOptions(ToggleableOptions { cursor_position: 0 }),
                MenuType::SelectableItems(SelectableItems {
                    items: vec![
                        ("ABOUT", MenuAction::SwitchMenu(5)),
                        ("CONTROLS", MenuAction::SwitchMenu(6)),
                        ("POWER-UPS", MenuAction::SwitchMenu(7)),
                    ],
                    cursor_position: 0,
                }),
                MenuType::HallOfFame,
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
                MenuType::ControlsScreen(
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
            battle_mode_sub_menu_state: None,
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

    pub fn get_enter_action(&self) -> MenuAction {
        match self.get_current_menu() {
            MenuType::SelectableItems(selectable_items) => selectable_items.get_action(),
            MenuType::ToggleableOptions(toggleable_options) => toggleable_options.get_action(),
            MenuType::StaticText(_) | MenuType::ControlsScreen(_) | MenuType::HallOfFame => {
                MenuAction::Back
            }
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

pub struct DemoModeStartTimer(pub Timer);
