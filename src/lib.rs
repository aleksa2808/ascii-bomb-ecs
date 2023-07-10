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

use bevy::{ecs as bevy_ecs, prelude::*, window::WindowResolution};
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[cfg(target_arch = "wasm32")]
    Loading,
    #[cfg(target_arch = "wasm32")]
    WebReadyToStart,
    SplashScreen,
    MainMenu,
    MapTransition,
    StoryModeSetup,
    StoryModeManager,
    BossSpeech,
    StoryModeInGame,
    HighScoreNameInput,
    StoryModeTeardown,
    BattleModeSetup,
    BattleModeManager,
    RoundStartFreeze,
    BattleModeInGame,
    LeaderboardDisplay,
    BattleModeTeardown,
    Paused,
    SecretModeSetup,
    SecretModeManager,
    SecretModeInGame,
    SecretModeTeardown,
}

impl Default for AppState {
    fn default() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                Self::Loading
            } else {
                // The loading state is not used in the native build in order to mimic
                // the original game's behavior (non-blocking splash screen)
                Self::SplashScreen
            }
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn run() {
    let mut app = App::new();

    #[cfg(not(target_arch = "wasm32"))]
    let resolution = WindowResolution::new(MENU_WIDTH as f32, MENU_HEIGHT as f32);
    #[cfg(target_arch = "wasm32")]
    let resolution = WindowResolution::new(MENU_WIDTH as f32, MENU_HEIGHT as f32)
        .with_scale_factor_override(1.0);

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "ascii-bomb-ecs".to_string(),
                    resolution,
                    resizable: false,
                    #[cfg(target_arch = "wasm32")]
                    canvas: Some("#bevy-canvas".to_string()),
                    ..Default::default()
                }),
                ..default()
            })
            // fixes blurry textures
            .set(ImagePlugin::default_nearest()),
    )
    .add_state::<AppState>()
    .add_plugin(AudioPlugin);

    #[cfg(target_arch = "wasm32")]
    app.add_plugin(LoadingPlugin {
        loading_state: AppState::Loading,
        next_state: AppState::WebReadyToStart,
    })
    .add_systems(
        (handle_web_input, apply_system_buffers)
            .chain()
            .in_set(crate::common::Label::InputMapping),
    )
    .add_system(web_ready_to_start_enter.in_schedule(OnEnter(AppState::WebReadyToStart)))
    .add_system(web_ready_to_start_update.in_set(OnUpdate(AppState::WebReadyToStart)));

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
