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
use wasm_bindgen::prelude::*;

use crate::{camera::SimpleOrthoProjection, constants::*, events::*, resources::*, systems::*};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    MainMenu,
    InGame,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
enum Label {
    Input,
    Explosion,
    Burn,
    Damage,
    Spawner,
}

#[wasm_bindgen]
pub fn run() {
    let mut app = App::build();

    const DISPLAY_SCALE: usize = 2;
    app.insert_resource(WindowDescriptor {
        title: "ascii-bomb-ecs".to_string(),
        height: (MAP_HEIGHT * DISPLAY_SCALE * TILE_HEIGHT) as f32,
        width: (MAP_WIDTH * DISPLAY_SCALE * TILE_WIDTH) as f32,
        ..Default::default()
    })
    .add_plugins(DefaultPlugins);

    // when building for Web, use WebGL2 rendering
    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);

    use bevy::render::camera::camera_system;

    app.add_state(AppState::MainMenu)
        .init_resource::<Fonts>()
        .init_resource::<ButtonMaterials>()
        .insert_resource(ClearColor(Color::BLACK))
        .add_event::<PlayerActionEvent>()
        .add_event::<ExplosionEvent>()
        .add_event::<BurnEvent>()
        .add_event::<DamageEvent>()
        .add_startup_system(load_textures.system())
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
        .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup_game.system()))
        .add_system_set(SystemSet::on_exit(AppState::InGame).with_system(teardown.system()))
        .add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(pop_state_on_esc.system())
                // display game stats
                .with_system(display_stats.system())
                // time effect update
                .with_system(move_cooldown_tick.system().before(Label::Input))
                .with_system(
                    perishable_tick
                        .system()
                        .before(Label::Explosion)
                        .label(Label::Spawner),
                )
                .with_system(immortality_tick.system())
                // handle input
                .with_system(handle_keyboard_input.system().label(Label::Input))
                .with_system(handle_mouse_input.system().label(Label::Input))
                // handle AI
                .with_system(mob_ai.system().label(Label::Input))
                .with_system(bot_ai.system().label(Label::Input))
                // handle movement
                .with_system(player_move.system().after(Label::Input))
                .with_system(moving_object_update.system())
                // handle bomb logic
                .with_system(bomb_drop.system().after(Label::Input).label(Label::Spawner))
                .with_system(
                    handle_explosion
                        .system()
                        .label(Label::Explosion)
                        .label(Label::Spawner),
                )
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
                .with_system(item_burn.system().label(Label::Burn).label(Label::Spawner))
                .with_system(exit_burn.system().label(Label::Burn).label(Label::Spawner))
                // player specifics
                .with_system(pick_up_item.system())
                .with_system(melee_attack.system().before(Label::Damage))
                .with_system(player_damage.system().label(Label::Damage))
                // animation
                .with_system(animate_fuse.system())
                .with_system(animate_immortality.system())
                // game end check
                // TODO: probably buggy right now since at the end of a level/game you need to remove all the entities
                // and what if someone registered an entity to be spawned in the same frame, will that entity spawn in
                // the next level / main menu?
                // previously this was in a separate stage, but that doesn't seem to play nice with states
                .with_system(finish_level.system().after(Label::Spawner))
                .with_system(fail_level.system().after(Label::Spawner)),
        );

    app.run();
}
