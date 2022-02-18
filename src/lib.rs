mod audio;
mod battle_mode;
mod common;
mod game;
mod loading;
mod main_menu;
mod map_transition;
mod secret_mode;
mod splash_screen;
mod story_mode;
#[cfg(target_arch = "wasm32")]
mod web;

use bevy::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::{
    audio::AudioPlugin,
    battle_mode::BattleModePlugin,
    common::CommonPlugin,
    game::GamePlugin,
    main_menu::{MainMenuPlugin, MENU_HEIGHT, MENU_WIDTH},
    map_transition::MapTransitionPlugin,
    secret_mode::SecretModePlugin,
    splash_screen::SplashScreenPlugin,
    story_mode::StoryModePlugin,
};
#[cfg(target_arch = "wasm32")]
use crate::{loading::LoadingPlugin, web::*};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[cfg(target_arch = "wasm32")]
    Loading,
    #[cfg(target_arch = "wasm32")]
    WebReadyToStart,
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

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn run() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        title: "ascii-bomb-ecs".to_string(),
        width: MENU_WIDTH as f32,
        height: MENU_HEIGHT as f32,
        resizable: false,
        #[cfg(target_arch = "wasm32")]
        canvas: Some("#bevy-canvas".to_string()),
        #[cfg(target_arch = "wasm32")]
        scale_factor_override: Some(1.0),
        ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    .add_plugin(AudioPlugin);

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            app.add_state(AppState::Loading)
                .add_plugin(LoadingPlugin {
                    loading_state: AppState::Loading,
                    next_state: AppState::WebReadyToStart,
                })
                .add_system(handle_web_input.exclusive_system().label(crate::common::Label::InputMapping))
                .add_system_set(
                    SystemSet::on_enter(AppState::WebReadyToStart).with_system(web_ready_to_start_enter),
                )
                .add_system_set(
                    SystemSet::on_update(AppState::WebReadyToStart).with_system(web_ready_to_start_update),
                );
        } else {
            // The loading state is not used in the native build in order to mimic
            // the original game's behavior (non-blocking splash screen)
            app.add_state(AppState::SplashScreen);
        }
    }

    app.add_plugin(CommonPlugin)
        .add_plugin(SplashScreenPlugin)
        .add_plugin(MainMenuPlugin)
        .add_plugin(GamePlugin)
        .add_plugin(MapTransitionPlugin)
        .add_plugin(StoryModePlugin)
        .add_plugin(BattleModePlugin)
        .add_plugin(SecretModePlugin);

    app.run();
}
