use std::time::Duration;

use bevy::{
    ecs as bevy_ecs,
    prelude::{Entity, Resource},
    time::{Timer, TimerMode},
};

use crate::game::components::Penguin;

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

#[derive(Resource)]
pub struct StoryModeContext {
    pub state: StoryModeState,
    pub level: Level,
    pub level_outcome: Option<LevelOutcome>,
    pub game_completed: bool,
}

// boss speech
#[derive(Resource)]
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
            line_timer: Timer::from_seconds(0.02, TimerMode::Repeating),
        }
    }

    pub fn tick(&mut self, delta: Duration) {
        self.line_timer.tick(delta);
        if self.line_in_progress() {
            self.line_character = (self.line_character
                + self.line_timer.times_finished_this_tick() as usize)
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

#[derive(Resource)]
pub struct BossSpeechBoxEntities {
    pub speech_box: Entity,
    pub speaker_portrait: Entity,
    pub speech_text: Entity,
}

#[derive(Resource)]
pub struct HighScoreNameInputContext {
    pub input_box: Entity,
    pub name_text: Entity,
}
