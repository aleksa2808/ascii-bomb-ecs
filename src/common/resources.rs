use std::{cmp::Reverse, fs};

use bevy::{
    ecs as bevy_ecs,
    prelude::*,
    utils::{HashMap, HashSet},
};
use serde::{Deserialize, Serialize};

use crate::loading::resources::LoadingAssetHandles;

use super::types::InputAction;

#[derive(Default, Resource)]
pub struct InputActionStatusTracker {
    active_actions: HashSet<InputAction>,
}

impl InputActionStatusTracker {
    pub fn set_active(&mut self, input_action: InputAction) {
        self.active_actions.insert(input_action);
    }

    pub fn is_active(&self, input_action: InputAction) -> bool {
        self.active_actions.contains(&input_action)
    }

    pub fn get_active(&self) -> &HashSet<InputAction> {
        &self.active_actions
    }

    pub fn clear(&mut self) {
        self.active_actions.clear();
    }
}

#[derive(Resource)]
pub struct Fonts {
    pub mono: Handle<Font>,
}

impl FromWorld for Fonts {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        let fonts = Fonts {
            mono: asset_server.load("fonts/UbuntuMono-R.ttf"),
        };

        if let Some(mut loading_asset_handles) = world.get_resource_mut::<LoadingAssetHandles>() {
            loading_asset_handles.0.push(fonts.mono.clone().untyped());
        }

        fonts
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

#[derive(Serialize, Deserialize, Resource)]
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
                let new = Self(
                    GameOption::LIST
                        .iter()
                        .map(|o| {
                            (
                                *o,
                                match o {
                                    GameOption::Sound => true,
                                    GameOption::Demo => true,
                                    GameOption::Transition => false,
                                },
                            )
                        })
                        .collect(),
                );
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
        match fs::create_dir_all(options_file_path.parent().unwrap()) {
            Ok(()) => fs::write(options_file_path, serialized).unwrap(),
            Err(e) => eprintln!("Cannot save options file. Error: {}", e),
        }
    }

    pub fn toggle(&mut self, option: GameOption) -> bool {
        let new_value = !self.get(option);
        self.0.insert(option, new_value);
        self.save();
        new_value
    }
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

#[derive(Serialize, Deserialize, Resource)]
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
        let high_scores_file_path = std::path::Path::new(Self::SERIALIZED_FILE_PATH);
        let serialized = serde_json::to_string(self).unwrap();
        match fs::create_dir_all(high_scores_file_path.parent().unwrap()) {
            Ok(()) => fs::write(high_scores_file_path, serialized).unwrap(),
            Err(e) => eprintln!("Cannot save high scores file. Error: {}", e),
        }
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
