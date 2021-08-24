mod camera;
mod components;
mod constants;
mod events;
mod item;
mod resources;
mod systems;
mod types;
mod utils;

use bevy::{prelude::*, window::exit_on_window_close_system};

use crate::{camera::SimpleOrthoProjection, constants::*, events::*, resources::*, systems::*};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    MainMenu,
    StoryMode,
    BattleMode,
    Paused,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
enum Label {
    TimeUpdate,
    Input,
    Explosion,
    Burn,
    Damage,
    GameEndCheck,
}

fn add_common_game_systems(app: &mut App, state: AppState) {
    app.add_system_set(SystemSet::on_exit(state).with_system(teardown))
        .add_system_set(
            SystemSet::on_update(state)
                // time effect update
                .with_system(move_cooldown_tick.label(Label::TimeUpdate))
                .with_system(
                    perishable_tick
                        .label(Label::TimeUpdate)
                        .before(Label::Explosion),
                )
                .with_system(game_timer_tick.label(Label::TimeUpdate))
                .with_system(immortality_tick.label(Label::TimeUpdate))
                // handle input
                .with_system(handle_keyboard_input.label(Label::Input))
                .with_system(handle_mouse_input.label(Label::Input))
                // handle AI
                .with_system(mob_ai.label(Label::Input))
                .with_system(bot_ai.label(Label::Input).after(Label::TimeUpdate))
                // handle movement
                .with_system(player_move.after(Label::Input).after(Label::TimeUpdate))
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
                .with_system(animate_immortality.after(Label::TimeUpdate))
                // display game stats
                .with_system(display_stats.after(Label::TimeUpdate).after(Label::Damage)),
        );
}

pub fn run() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        title: "ascii-bomb-ecs".to_string(),
        resizable: false,
        ..Default::default()
    })
    .add_plugins(DefaultPlugins);

    use bevy::render::camera::camera_system;

    app.add_state(AppState::MainMenu)
        .init_resource::<MenuMaterials>()
        .init_resource::<MenuState>()
        .init_resource::<Fonts>()
        .init_resource::<Textures>()
        .insert_resource(ClearColor(COLORS[0].into()))
        .add_event::<PlayerActionEvent>()
        .add_event::<ExplosionEvent>()
        .add_event::<BurnEvent>()
        .add_event::<DamageEvent>()
        .add_system(exit_on_window_close_system)
        .add_system_to_stage(
            CoreStage::PostUpdate,
            camera_system::<SimpleOrthoProjection>,
        )
        .add_system_set(SystemSet::on_enter(AppState::MainMenu).with_system(setup_menu))
        .add_system_set(SystemSet::on_resume(AppState::MainMenu).with_system(setup_menu))
        .add_system_set(SystemSet::on_pause(AppState::MainMenu).with_system(teardown))
        .add_system_set(SystemSet::on_exit(AppState::MainMenu).with_system(teardown))
        .add_system_set(SystemSet::on_update(AppState::MainMenu).with_system(menu))
        .add_system_set(
            SystemSet::on_update(AppState::Paused)
                .with_system(display_stats)
                .with_system(pop_state_on_enter),
        );

    add_common_game_systems(&mut app, AppState::StoryMode);
    app.add_system_set(SystemSet::on_enter(AppState::StoryMode).with_system(setup_story_mode))
        .add_system_set(
            SystemSet::on_update(AppState::StoryMode)
                // game end check
                .with_system(
                    fail_level
                        .exclusive_system()
                        .at_end()
                        .label(Label::GameEndCheck),
                )
                .with_system(
                    finish_level
                        .exclusive_system()
                        .at_end()
                        .label(Label::GameEndCheck),
                ),
        );

    add_common_game_systems(&mut app, AppState::BattleMode);
    app.add_system_set(SystemSet::on_enter(AppState::BattleMode).with_system(setup_battle_mode))
        .add_system_set(
            SystemSet::on_update(AppState::BattleMode)
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
                ),
        );

    app.run();
}
