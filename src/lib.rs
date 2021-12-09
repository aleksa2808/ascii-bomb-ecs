mod battle_mode;
mod common;
mod game;
mod main_menu;
mod map_transition;
mod secret_mode;
mod splash_screen;
mod story_mode;

use bevy::prelude::*;
use bevy_kira_audio::AudioPlugin;
use wasm_bindgen::prelude::*;

use crate::{
    battle_mode::BattleModePlugin,
    common::CommonPlugin,
    game::GamePlugin,
    main_menu::{MainMenuPlugin, MENU_HEIGHT, MENU_WIDTH},
    map_transition::MapTransitionPlugin,
    secret_mode::SecretModePlugin,
    splash_screen::SplashScreenPlugin,
    story_mode::StoryModePlugin,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    SplashScreen,
    MainMenu,
    MapTransition,
    StoryMode,
    BossSpeech,
    StoryModeInGame,
    HighScoreNameInput,
    BattleMode,
    RoundStartFreeze,
    BattleModeInGame,
    LeaderboardDisplay,
    Paused,
    SecretMode,
    SecretModeInGame,
}

#[wasm_bindgen]
pub fn run() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        title: "ascii-bomb-ecs".to_string(),
        width: MENU_WIDTH as f32,
        height: MENU_HEIGHT as f32,
        resizable: false,
        #[cfg(target_arch = "wasm32")]
        canvas: Some("#bevy-canvas".to_string()),
        ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    .add_plugin(AudioPlugin);

    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);

    app.add_state(AppState::SplashScreen)
        .add_plugin(CommonPlugin)
        .add_plugin(SplashScreenPlugin)
        .add_plugin(MainMenuPlugin)
        .add_plugin(GamePlugin)
        .add_plugin(MapTransitionPlugin)
        .add_plugin(StoryModePlugin)
        .add_plugin(BattleModePlugin)
        .add_plugin(SecretModePlugin);

    app.run();
}
