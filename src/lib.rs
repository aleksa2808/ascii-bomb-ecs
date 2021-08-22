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
}

fn add_common_game_systems(app: &mut App, state: AppState) {
    app.add_system_set(SystemSet::on_exit(state).with_system(teardown.system()))
        .add_system_set(
            SystemSet::on_update(state)
                // time effect update
                .with_system(move_cooldown_tick.system().label(Label::TimeUpdate))
                .with_system(
                    perishable_tick
                        .system()
                        .label(Label::TimeUpdate)
                        .before(Label::Explosion),
                )
                .with_system(game_timer_tick.system().label(Label::TimeUpdate))
                .with_system(immortality_tick.system().label(Label::TimeUpdate))
                // handle input
                .with_system(handle_keyboard_input.system().label(Label::Input))
                .with_system(handle_mouse_input.system().label(Label::Input))
                // handle AI
                .with_system(mob_ai.system().label(Label::Input))
                .with_system(bot_ai.system().label(Label::Input).after(Label::TimeUpdate))
                // handle movement
                .with_system(
                    player_move
                        .system()
                        .after(Label::Input)
                        .after(Label::TimeUpdate),
                )
                .with_system(moving_object_update.system())
                // handle bomb logic
                .with_system(bomb_drop.system().after(Label::Input))
                .with_system(handle_explosion.system().label(Label::Explosion))
                .with_system(
                    fire_effect
                        .system()
                        .after(Label::Explosion)
                        .before(Label::Burn),
                )
                .with_system(
                    player_burn
                        .system()
                        .label(Label::Burn)
                        .before(Label::Damage),
                )
                .with_system(bomb_burn.system().label(Label::Burn))
                .with_system(destructible_wall_burn.system().label(Label::Burn))
                .with_system(item_burn.system().label(Label::Burn))
                .with_system(exit_burn.system().label(Label::Burn))
                // player specifics
                .with_system(pick_up_item.system())
                .with_system(melee_attack.system().before(Label::Damage))
                .with_system(player_damage.system().label(Label::Damage))
                // animation
                .with_system(animate_fuse.system().after(Label::TimeUpdate))
                .with_system(animate_immortality.system().after(Label::TimeUpdate))
                // display game stats
                .with_system(
                    display_stats
                        .system()
                        .after(Label::TimeUpdate)
                        .after(Label::Damage),
                )
                // game end check
                .with_system(fail_level.exclusive_system().at_end()),
        );
}

pub fn run() {
    let mut app = App::new();

    const DISPLAY_SCALE: usize = 2;
    app.insert_resource(WindowDescriptor {
        title: "ascii-bomb-ecs".to_string(),
        height: (MAP_HEIGHT * DISPLAY_SCALE * TILE_HEIGHT) as f32,
        width: (MAP_WIDTH * DISPLAY_SCALE * TILE_WIDTH) as f32,
        ..Default::default()
    })
    .add_plugins(DefaultPlugins);

    use bevy::render::camera::camera_system;

    app.add_state(AppState::MainMenu)
        .init_resource::<Textures>()
        .init_resource::<Fonts>()
        .init_resource::<ButtonMaterials>()
        .insert_resource(ClearColor(Color::BLACK))
        .add_event::<PlayerActionEvent>()
        .add_event::<ExplosionEvent>()
        .add_event::<BurnEvent>()
        .add_event::<DamageEvent>()
        .add_system(exit_on_window_close_system.system())
        .add_system_to_stage(
            CoreStage::PostUpdate,
            camera_system::<SimpleOrthoProjection>.system(),
        )
        .add_system_set(SystemSet::on_enter(AppState::MainMenu).with_system(setup_menu.system()))
        .add_system_set(SystemSet::on_resume(AppState::MainMenu).with_system(setup_menu.system()))
        .add_system_set(SystemSet::on_pause(AppState::MainMenu).with_system(teardown.system()))
        .add_system_set(SystemSet::on_exit(AppState::MainMenu).with_system(teardown.system()))
        .add_system_set(
            SystemSet::on_update(AppState::MainMenu)
                .with_system(enter_game_on_enter.system())
                .with_system(menu.system())
                .with_system(exit_on_esc.system()),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Paused)
                .with_system(display_stats.system())
                .with_system(pop_state_on_enter.system()),
        );

    add_common_game_systems(&mut app, AppState::StoryMode);
    app.add_system_set(
        SystemSet::on_enter(AppState::StoryMode).with_system(setup_story_mode.system()),
    )
    .add_system_set(
        SystemSet::on_update(AppState::StoryMode)
            .with_system(finish_level.exclusive_system().at_end()),
    );

    add_common_game_systems(&mut app, AppState::BattleMode);
    app.add_system_set(
        SystemSet::on_enter(AppState::BattleMode).with_system(setup_battle_mode.system()),
    );

    app.run();
}
