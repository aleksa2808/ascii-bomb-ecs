use bevy::{ecs as bevy_ecs, prelude::*, utils::HashMap};

use crate::{
    audio::{SoundHandles, SoundID},
    common::{constants::COLORS, resources::GameOption},
    game::types::BotDifficulty,
    loading::resources::AssetsLoading,
};

#[derive(Resource)]
pub struct MenuColors {
    pub background_color: Color,
    pub modal_background_color: Color,
    pub modal_foreground_color: Color,
}

impl Default for MenuColors {
    fn default() -> Self {
        Self {
            background_color: COLORS[0].into(),
            modal_background_color: COLORS[8].into(),
            modal_foreground_color: COLORS[1].into(),
        }
    }
}

#[derive(Resource)]
pub struct MainMenuSoundEffects {
    pub confirm: SoundID,
    pub select: SoundID,
}

impl FromWorld for MainMenuSoundEffects {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        let confirm_handle = asset_server.load("sounds/confirm.wav");
        let select_handle = asset_server.load("sounds/select.wav");

        if let Some(mut assets_loading) = world.get_resource_mut::<AssetsLoading>() {
            assets_loading.0.append(&mut vec![
                confirm_handle.clone_untyped(),
                select_handle.clone_untyped(),
            ]);
        }

        let mut sound_handles = world.get_resource_mut::<SoundHandles>().unwrap();
        Self {
            confirm: sound_handles.add_handle(confirm_handle),
            select: sound_handles.add_handle(select_handle),
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
    BotDifficulty,
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

pub struct AmountOfActors {
    amount_of_players: ConfigurationOption<usize>,
    amount_of_bots: ConfigurationOption<usize>,
}

impl AmountOfActors {
    const MIN_AMOUNT_OF_PLAYERS: usize = 1;
    const MIN_AMOUNT_OF_BOTS: usize = 0;
    const MIN_AMOUNT_OF_ACTORS: usize = 2;

    const MAX_AMOUNT_OF_PLAYERS: usize = 2;
    const MAX_AMOUNT_OF_ACTORS: usize = 8;

    fn new(
        initial_amount_of_players: usize,
        initial_amount_of_bots: usize,
    ) -> Result<Self, String> {
        if initial_amount_of_players < Self::MIN_AMOUNT_OF_PLAYERS {
            return Err(format!(
                "Initial amount of players ({}) is less than the minimum ({}).",
                initial_amount_of_players,
                Self::MIN_AMOUNT_OF_PLAYERS
            ));
        }

        // TODO: check overflow edge cases
        if initial_amount_of_players + initial_amount_of_bots < Self::MIN_AMOUNT_OF_ACTORS {
            return Err(format!(
                "Initial amount of players and bots ({}) is less than the minimum ({}).",
                initial_amount_of_players + initial_amount_of_bots,
                Self::MIN_AMOUNT_OF_ACTORS
            ));
        }

        if initial_amount_of_players > Self::MAX_AMOUNT_OF_PLAYERS {
            return Err(format!(
                "Initial number of players ({}) exceeds the maximum ({}).",
                initial_amount_of_players,
                Self::MAX_AMOUNT_OF_PLAYERS
            ));
        }

        // TODO: check overflow edge cases
        if initial_amount_of_players + initial_amount_of_bots > Self::MAX_AMOUNT_OF_ACTORS {
            return Err(format!(
                "Initial number of players and bots ({}) exceeds the maximum ({}).",
                initial_amount_of_players + initial_amount_of_bots,
                Self::MAX_AMOUNT_OF_ACTORS
            ));
        }

        Ok(Self {
            amount_of_players: ConfigurationOption::new(
                (Self::MIN_AMOUNT_OF_PLAYERS..=Self::MAX_AMOUNT_OF_PLAYERS).collect(),
                initial_amount_of_players - Self::MIN_AMOUNT_OF_PLAYERS,
            )
            .unwrap(),
            amount_of_bots: ConfigurationOption::new(
                (Self::MIN_AMOUNT_OF_BOTS
                    ..=Self::MAX_AMOUNT_OF_ACTORS - Self::MIN_AMOUNT_OF_PLAYERS)
                    .collect(),
                initial_amount_of_bots - Self::MIN_AMOUNT_OF_BOTS,
            )
            .unwrap(),
        })
    }

    fn can_increase_amount(&self) -> bool {
        self.amount_of_players() + self.amount_of_bots() < Self::MAX_AMOUNT_OF_ACTORS
    }

    fn can_decrease_amount(&self) -> bool {
        self.amount_of_players() + self.amount_of_bots() > Self::MIN_AMOUNT_OF_ACTORS
    }

    pub fn amount_of_players(&self) -> usize {
        *self.amount_of_players.value()
    }

    pub fn increment_amount_of_players(&mut self) {
        if self.amount_of_players() < Self::MAX_AMOUNT_OF_PLAYERS && self.can_increase_amount() {
            self.amount_of_players.increment();
        }
    }

    pub fn decrement_amount_of_players(&mut self) {
        if self.amount_of_players() > Self::MIN_AMOUNT_OF_PLAYERS && self.can_decrease_amount() {
            self.amount_of_players.decrement();
        }
    }

    pub fn amount_of_bots(&self) -> usize {
        *self.amount_of_bots.value()
    }

    pub fn increment_amount_of_bots(&mut self) {
        if self.can_increase_amount() {
            self.amount_of_bots.increment();
        }
    }

    pub fn decrement_amount_of_bots(&mut self) {
        if self.amount_of_bots() > Self::MIN_AMOUNT_OF_BOTS && self.can_decrease_amount() {
            self.amount_of_bots.decrement();
        }
    }
}

pub struct BattleModeSubMenuState {
    pub amount_of_actors: AmountOfActors,
    pub step: BattleModeSubMenuStep,
    pub winning_score: ConfigurationOption<usize>,
    pub difficulty: ConfigurationOption<BotDifficulty>,
}

impl Default for BattleModeSubMenuState {
    fn default() -> Self {
        Self {
            step: BattleModeSubMenuStep::AmountOfPlayers,
            amount_of_actors: AmountOfActors::new(1, 3).unwrap(),
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

#[derive(Resource)]
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

#[derive(Resource)]
pub struct MenuBackgroundAnimationContext {
    pub entity_change_parameters: HashMap<Entity, MenuBackgroundEntityChangeParameters>,
    pub timer: Timer,
}

#[derive(Resource)]
pub struct DemoModeStartTimer(pub Timer);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_of_actors_init() {
        assert!(AmountOfActors::new(1, 1).is_ok());
        assert!(AmountOfActors::new(2, 0).is_ok());
        assert!(AmountOfActors::new(2, 6).is_ok());
        assert!(AmountOfActors::new(1, 7).is_ok());

        assert!(AmountOfActors::new(0, 0).is_err());
        assert!(AmountOfActors::new(0, 1).is_err());
        assert!(AmountOfActors::new(0, 2).is_err());
        assert!(AmountOfActors::new(1, 0).is_err());
        assert!(AmountOfActors::new(1, 8).is_err());
        assert!(AmountOfActors::new(3, 0).is_err());
        assert!(AmountOfActors::new(2, 7).is_err());
    }

    #[test]
    fn test_amount_of_actors_modification() {
        let mut amount_of_actors = AmountOfActors::new(1, 1).unwrap();
        assert_eq!(amount_of_actors.amount_of_players(), 1);
        assert_eq!(amount_of_actors.amount_of_bots(), 1);

        // at the minimum amount of actors, so this shouldn't do anything
        amount_of_actors.decrement_amount_of_bots();
        assert_eq!(amount_of_actors.amount_of_players(), 1);
        assert_eq!(amount_of_actors.amount_of_bots(), 1);

        amount_of_actors.increment_amount_of_players();
        amount_of_actors.decrement_amount_of_bots();
        assert_eq!(amount_of_actors.amount_of_players(), 2);
        assert_eq!(amount_of_actors.amount_of_bots(), 0);

        // at the maximum amount of actors, so this shouldn't do anything
        amount_of_actors.increment_amount_of_players();
        assert_eq!(amount_of_actors.amount_of_players(), 2);
        assert_eq!(amount_of_actors.amount_of_bots(), 0);

        amount_of_actors.increment_amount_of_bots();
        amount_of_actors.increment_amount_of_bots();
        amount_of_actors.increment_amount_of_bots();
        amount_of_actors.increment_amount_of_bots();
        amount_of_actors.increment_amount_of_bots();
        amount_of_actors.increment_amount_of_bots();
        assert_eq!(amount_of_actors.amount_of_players(), 2);
        assert_eq!(amount_of_actors.amount_of_bots(), 6);

        // at the maximum amount of actors, so this shouldn't do anything
        amount_of_actors.increment_amount_of_bots();
        assert_eq!(amount_of_actors.amount_of_players(), 2);
        assert_eq!(amount_of_actors.amount_of_bots(), 6);

        amount_of_actors.decrement_amount_of_players();
        amount_of_actors.increment_amount_of_bots();
        assert_eq!(amount_of_actors.amount_of_players(), 1);
        assert_eq!(amount_of_actors.amount_of_bots(), 7);
    }
}
