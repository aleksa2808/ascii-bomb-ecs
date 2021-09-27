mod camera;
mod components;
mod constants;
mod events;
mod item;
mod main_menu;
mod map_transition;
mod resources;
mod splash_screen;
mod systems;
mod types;
mod utils;

use bevy::{prelude::*, render::camera::camera_system, window::exit_on_window_close_system};
use bevy_kira_audio::AudioPlugin;

use crate::{
    camera::SimpleOrthoProjection,
    events::*,
    main_menu::{MainMenuPlugin, MENU_HEIGHT, MENU_WIDTH},
    map_transition::MapTransitionPlugin,
    resources::*,
    splash_screen::SplashScreenPlugin,
    systems::*,
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

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
enum Label {
    Setup,
    TimeUpdate,
    Input,
    PlayerMovement,
    Explosion,
    Burn,
    Damage,
    GameEndCheck,
}

fn add_common_game_systems(app: &mut App, state: AppState) {
    app.add_system_set(SystemSet::on_enter(state).with_system(setup_penguin_portraits))
        .add_system_set(
            SystemSet::on_update(state)
                // time effect update
                .with_system(move_cooldown_tick.label(Label::TimeUpdate))
                .with_system(
                    perishable_tick
                        .label(Label::TimeUpdate)
                        .before(Label::Explosion),
                )
                .with_system(immortality_tick.label(Label::TimeUpdate))
                // handle input
                .with_system(handle_keyboard_input.label(Label::Input))
                .with_system(handle_mouse_input.label(Label::Input))
                // handle AI
                .with_system(mob_ai.label(Label::Input))
                .with_system(bot_ai.label(Label::Input).after(Label::TimeUpdate))
                // handle movement
                .with_system(
                    player_move
                        .label(Label::PlayerMovement)
                        .after(Label::Input)
                        .after(Label::TimeUpdate),
                )
                .with_system(moving_object_update)
                // handle bomb logic
                .with_system(bomb_drop.after(Label::Input))
                .with_system(handle_explosion.label(Label::Explosion))
                .with_system(fire_effect.after(Label::Explosion).before(Label::Burn))
                .with_system(player_burn.label(Label::Burn).before(Label::Damage))
                .with_system(bomb_burn.label(Label::Burn))
                .with_system(destructible_wall_burn.label(Label::Burn))
                .with_system(item_burn.label(Label::Burn))
                .with_system(exit_burn.label(Label::Burn))
                // player specifics
                .with_system(pick_up_item)
                .with_system(melee_attack.before(Label::Damage))
                .with_system(player_damage.label(Label::Damage))
                // animation
                .with_system(animate_fuse.after(Label::TimeUpdate))
                .with_system(animate_immortality.after(Label::TimeUpdate)),
        );
}

pub fn run() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        title: "ascii-bomb-ecs".to_string(),
        width: MENU_WIDTH as f32,
        height: MENU_HEIGHT as f32,
        resizable: false,
        ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    .add_plugin(AudioPlugin);

    app.add_state(AppState::SplashScreen)
        .add_plugin(SplashScreenPlugin)
        .add_plugin(MainMenuPlugin)
        .add_plugin(MapTransitionPlugin)
        .init_resource::<BaseColorMaterials>()
        .init_resource::<GameOptionStore>()
        .init_resource::<PersistentHighScores>()
        .init_resource::<Fonts>()
        .init_resource::<HUDMaterials>()
        .init_resource::<Textures>()
        .init_resource::<Sounds>()
        .add_event::<PlayerActionEvent>()
        .add_event::<ExplosionEvent>()
        .add_event::<BurnEvent>()
        .add_event::<DamageEvent>()
        .add_system(exit_on_window_close_system)
        .add_startup_system(set_volume_based_on_options)
        .add_system_to_stage(
            CoreStage::PostUpdate,
            camera_system::<SimpleOrthoProjection>,
        )
        .add_system_set(
            SystemSet::on_update(AppState::Paused)
                .with_system(hud_update)
                .with_system(pop_state_on_enter),
        );

    app.add_system_set(
        SystemSet::on_enter(AppState::StoryMode)
            .with_system(setup_story_mode.exclusive_system().label(Label::Setup))
            .with_system(resize_window.exclusive_system().after(Label::Setup))
            .with_system(spawn_cameras.exclusive_system().after(Label::Setup)),
    )
    .add_system_set(SystemSet::on_exit(AppState::StoryMode).with_system(teardown))
    .add_system_set(SystemSet::on_update(AppState::StoryMode).with_system(story_mode_dispatch))
    .add_system_set(SystemSet::on_enter(AppState::BossSpeech).with_system(setup_boss_speech))
    .add_system_set(SystemSet::on_update(AppState::BossSpeech).with_system(boss_speech_update))
    .add_system_set(
        SystemSet::on_enter(AppState::HighScoreNameInput).with_system(setup_high_score_name_input),
    )
    .add_system_set(
        SystemSet::on_update(AppState::HighScoreNameInput)
            .with_system(high_score_name_input_update),
    );

    add_common_game_systems(&mut app, AppState::StoryModeInGame);
    app.add_system_set(
        SystemSet::on_update(AppState::StoryModeInGame)
            .with_system(game_timer_tick.label(Label::TimeUpdate))
            // game end check
            .with_system(
                finish_level
                    .exclusive_system()
                    .at_end()
                    .label(Label::GameEndCheck),
            )
            // update HUD
            .with_system(
                hud_update
                    .exclusive_system()
                    .at_end()
                    .after(Label::GameEndCheck),
            ),
    );

    app.add_system_set(
        SystemSet::on_enter(AppState::BattleMode)
            .with_system(setup_battle_mode.exclusive_system().label(Label::Setup))
            .with_system(resize_window.exclusive_system().after(Label::Setup))
            .with_system(spawn_cameras.exclusive_system().after(Label::Setup)),
    )
    .add_system_set(SystemSet::on_exit(AppState::BattleMode).with_system(teardown))
    .add_system_set(SystemSet::on_update(AppState::BattleMode).with_system(battle_mode_dispatch))
    .add_system_set(SystemSet::on_update(AppState::RoundStartFreeze).with_system(finish_freeze))
    .add_system_set(
        SystemSet::on_enter(AppState::LeaderboardDisplay).with_system(setup_leaderboard_display),
    )
    .add_system_set(
        SystemSet::on_update(AppState::LeaderboardDisplay).with_system(leaderboard_display_update),
    );

    add_common_game_systems(&mut app, AppState::BattleModeInGame);
    app.add_system_set(
        SystemSet::on_enter(AppState::BattleModeInGame).with_system(trigger_round_start_freeze),
    )
    .add_system_set(
        SystemSet::on_update(AppState::BattleModeInGame)
            .with_system(game_timer_tick.label(Label::TimeUpdate))
            .with_system(
                wall_of_death_update
                    .exclusive_system()
                    .at_end()
                    .before(Label::GameEndCheck),
            )
            .with_system(
                finish_round
                    .exclusive_system()
                    .at_end()
                    .label(Label::GameEndCheck),
            )
            // update HUD
            .with_system(
                hud_update
                    .exclusive_system()
                    .at_end()
                    .after(Label::GameEndCheck),
            ),
    );

    app.add_system_set(
        SystemSet::on_enter(AppState::SecretMode)
            .with_system(setup_secret_mode.exclusive_system().label(Label::Setup))
            .with_system(resize_window.exclusive_system().after(Label::Setup))
            .with_system(spawn_cameras.exclusive_system().after(Label::Setup)),
    )
    .add_system_set(SystemSet::on_update(AppState::SecretMode).with_system(secret_mode_dispatch))
    .add_system_set(
        SystemSet::on_exit(AppState::SecretMode)
            .with_system(stop_audio)
            .with_system(teardown),
    );

    add_common_game_systems(&mut app, AppState::SecretModeInGame);
    app.add_system_set(
        SystemSet::on_update(AppState::SecretModeInGame)
            .with_system(update_secret_mode.exclusive_system().at_start())
            .with_system(
                finish_secret_mode
                    .after(Label::PlayerMovement)
                    .before(Label::Explosion),
            ),
    );

    app.run();
}
