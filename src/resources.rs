use std::{cmp::Reverse, collections::HashMap, fs, time::Duration};

use bevy::prelude::*;
use bevy_kira_audio::AudioSource;
use serde::{Deserialize, Serialize};

use crate::{
    components::{Penguin, Position},
    constants::COLORS,
    types::{BotDifficulty, Cooldown, Direction, PenguinControlType},
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameOption {
    Sound,
    Demo,
    Transition,
}

impl GameOption {
    pub const LIST: [GameOption; 3] = [GameOption::Sound, GameOption::Demo, GameOption::Transition];
}

#[derive(Serialize, Deserialize)]
pub struct GameOptionStore(HashMap<GameOption, bool>);

impl Default for GameOptionStore {
    fn default() -> Self {
        let options_file_path = std::path::Path::new(Self::OPTIONS_FILE_PATH);
        let game_option_store = if let Ok(content) = fs::read_to_string(options_file_path) {
            if let Ok(game_option_store) = serde_json::from_str(&content) {
                Some(game_option_store)
            } else {
                None
            }
        } else {
            None
        };

        match game_option_store {
            Some(game_option_store) => game_option_store,
            None => {
                println!("Missing or invalid options file detected, generating a new one.");
                let new = Self(GameOption::LIST.iter().map(|o| (*o, true)).collect());
                new.save();
                new
            }
        }
    }
}

impl GameOptionStore {
    const OPTIONS_FILE_PATH: &'static str = "local/options";

    pub fn get(&self, option: GameOption) -> bool {
        *self.0.get(&option).unwrap()
    }

    fn save(&self) {
        let options_file_path = std::path::Path::new(Self::OPTIONS_FILE_PATH);
        let serialized = serde_json::to_string(self).unwrap();
        fs::create_dir_all(options_file_path.parent().unwrap()).unwrap();
        fs::write(options_file_path, serialized).unwrap();
    }

    pub fn toggle(&mut self, option: GameOption) -> bool {
        let new_value = !self.get(option);
        self.0.insert(option, new_value);
        self.save();
        new_value
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

// story mode
pub enum StoryModeState {
    LevelSetup,
    MapTransition,
    BossSpeech,
    InGame,
    ScoreCheck,
    HighScoreNameInput,
}

#[derive(Clone, Copy)]
pub enum Level {
    Regular(usize),
    BossRoom,
}

pub enum LevelOutcome {
    Win,
    Loss,
}

pub struct GameScore(pub usize);

pub struct StoryModeContext {
    pub state: StoryModeState,
    pub level: Level,
    pub level_outcome: Option<LevelOutcome>,
    pub game_score: GameScore,
    pub game_completed: bool,
}

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

// high scores
#[derive(Serialize, Deserialize)]
pub struct HighScores(pub Vec<(String, usize)>);

impl HighScores {
    pub const HIGH_SCORES_MAX_SIZE: usize = 10;

    pub fn entry_threshold(&self) -> usize {
        if let Some(score) = self.0.get(Self::HIGH_SCORES_MAX_SIZE - 1) {
            score.1
        } else {
            0
        }
    }

    fn insert_score(&mut self, name: String, score: usize) {
        self.0.push((name, score));
        self.0.sort_by_key(|k| Reverse(k.1));
        self.0.truncate(Self::HIGH_SCORES_MAX_SIZE);
    }
}

#[derive(Serialize, Deserialize)]
pub struct PersistentHighScores {
    high_scores: HighScores,
    checksum: usize,
}

impl Default for PersistentHighScores {
    fn default() -> Self {
        let persistent_high_scores_file_path = std::path::Path::new(Self::SERIALIZED_FILE_PATH);
        let persistent_high_scores = if let Ok(content) =
            fs::read_to_string(persistent_high_scores_file_path)
        {
            if let Ok(persistent_high_scores) =
                serde_json::from_str::<PersistentHighScores>(&content)
            {
                if Self::calculate_checksum(&persistent_high_scores.high_scores) == persistent_high_scores.checksum
                && persistent_high_scores.high_scores.0.len() <= HighScores::HIGH_SCORES_MAX_SIZE
                // check if the scores are correctly sorted
                && persistent_high_scores.high_scores.0.windows(2).all(|w| w[0].1 >= w[1].1)
                {
                    Some(persistent_high_scores)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        match persistent_high_scores {
            Some(persistent_high_scores) => persistent_high_scores,
            None => {
                println!("Missing or invalid high scores file detected, generating a new one.");
                let persistent_high_scores = Self {
                    high_scores: HighScores(vec![]),
                    checksum: 0,
                };
                persistent_high_scores.save();
                persistent_high_scores
            }
        }
    }
}

impl PersistentHighScores {
    const SERIALIZED_FILE_PATH: &'static str = "local/high_scores";

    pub fn get_raw_scores(&self) -> &HighScores {
        &self.high_scores
    }

    pub fn entry_threshold(&self) -> usize {
        self.high_scores.entry_threshold()
    }

    pub fn insert_score(&mut self, name: String, score: usize) {
        if score > self.high_scores.entry_threshold() {
            self.high_scores.insert_score(name, score);
            self.checksum = Self::calculate_checksum(&self.high_scores);
            self.save();
        }
    }

    fn save(&self) {
        let options_file_path = std::path::Path::new(Self::SERIALIZED_FILE_PATH);
        let serialized = serde_json::to_string(self).unwrap();
        fs::create_dir_all(options_file_path.parent().unwrap()).unwrap();
        fs::write(options_file_path, serialized).unwrap();
    }

    fn calculate_checksum(high_scores: &HighScores) -> usize {
        let mut j = 1;
        let mut sum = 0;

        for score in &high_scores.0 {
            for ch in score.0.chars() {
                sum += j * ch as usize;
                j += 1;
            }

            let mut val = score.1;
            while val > 0 {
                sum += j * val % 10;
                j += 1;
                val /= 10;
            }
        }

        sum
    }
}

pub struct HighScoreNameInputContext {
    pub input_box: Entity,
    pub name_text: Entity,
}

// battle mode
pub struct BattleModeConfiguration {
    pub amount_of_players: usize,
    pub amount_of_bots: usize,
    pub winning_score: usize,
    pub difficulty: BotDifficulty,
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
    pub leaderboard: Leaderboard,
    pub round_outcome: Option<RoundOutcome>,
    // cache of the map block fill ratio
    pub percent_of_passable_positions_to_fill: f32,
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

// round start freeze
pub struct FreezeTimer(pub Timer);

// leaderboard display
pub struct LeaderboardDisplayContext {
    pub leaderboard_display_box: Entity,
    pub timer: Timer,
}

// secret mode
pub enum SecretModeDispatcherState {
    Setup,
    MapTransition,
    InGame,
}

pub enum SecretModeInGameState {
    Initial(Timer),
    Started {
        move_cooldown: Cooldown,
        round_progress: usize,
        round: usize,
    },
    Stopping(Timer),
}

pub struct SecretModeContext {
    pub dispatcher_state: SecretModeDispatcherState,
    pub in_game_state: SecretModeInGameState,
    pub pattern: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_scores_insert_score() {
        let mut high_scores = HighScores(vec![]);
        assert!(high_scores.0.is_empty());

        high_scores.insert_score(String::from("1"), 1);
        assert_eq!(high_scores.0.len(), 1);

        // if we insert more than allowed...
        for i in 0..2 * HighScores::HIGH_SCORES_MAX_SIZE {
            high_scores.insert_score(i.to_string(), i);
        }
        // ...we should not go over the allowed size
        assert_eq!(high_scores.0.len(), HighScores::HIGH_SCORES_MAX_SIZE);

        // check if the vector is still sorted after the new insertions
        assert!(high_scores.0.windows(2).all(|w| w[0].1 >= w[1].1));
    }

    #[test]
    fn test_high_scores_insert_score_ordering() {
        let mut high_scores = HighScores(vec![]);
        assert!(high_scores.0.is_empty());

        // if alice and bob have the same score but alice got it first...
        high_scores.insert_score(String::from("calvin"), 10);
        high_scores.insert_score(String::from("alice"), 1234);
        high_scores.insert_score(String::from("bob"), 1234);
        // ...then alice should be ahead of bob in the list
        assert_eq!(high_scores.0.len(), 3);
        assert_eq!(high_scores.0[0].0, "alice");
        assert_eq!(high_scores.0[1].0, "bob");
        assert_eq!(high_scores.0[2].0, "calvin");

        let mut high_scores = HighScores(vec![]);
        assert!(high_scores.0.is_empty());

        // if alice and bob have the same score but bob got it first...
        high_scores.insert_score(String::from("bob"), 1234);
        high_scores.insert_score(String::from("calvin"), 10000);
        high_scores.insert_score(String::from("alice"), 1234);
        // ...then bob should be ahead of alice in the list
        assert_eq!(high_scores.0.len(), 3);
        assert_eq!(high_scores.0[0].0, "calvin");
        assert_eq!(high_scores.0[1].0, "bob");
        assert_eq!(high_scores.0[2].0, "alice");
    }

    #[test]
    fn test_high_scores_entry_threshold() {
        let mut high_scores = HighScores(vec![]);
        assert!(high_scores.0.is_empty());
        assert_eq!(high_scores.entry_threshold(), 0);

        // if we insert one score short of the allowed size...
        let really_high_score = 1000000;
        for _ in 0..HighScores::HIGH_SCORES_MAX_SIZE - 1 {
            high_scores.insert_score(really_high_score.to_string(), really_high_score);
        }
        // ...there should still be room for more
        assert_eq!(high_scores.0.len(), HighScores::HIGH_SCORES_MAX_SIZE - 1);
        assert_eq!(high_scores.entry_threshold(), 0);

        // if we insert one more...
        high_scores.insert_score(really_high_score.to_string(), really_high_score);
        // ...the next score will need to be higher than the lowest one
        assert_eq!(high_scores.0.len(), HighScores::HIGH_SCORES_MAX_SIZE);
        assert_eq!(high_scores.entry_threshold(), really_high_score);
    }
}
