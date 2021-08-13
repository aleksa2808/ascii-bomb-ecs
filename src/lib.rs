mod camera;
mod components;
mod constants;
mod events;
mod item;
mod resources;
mod systems;
mod types;
mod utils;

use bevy::{input::system::exit_on_esc_system, prelude::*, window::exit_on_window_close_system};
use wasm_bindgen::prelude::*;

use crate::{
    camera::SimpleOrthoProjection, constants::*, events::*, resources::*, systems::*, types::*,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
enum Stage {
    GameEndCheck,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
enum Label {
    Input,
    Explosion,
    Burn,
    Damage,
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

    app.add_stage_after(
        CoreStage::Update,
        Stage::GameEndCheck,
        SystemStage::parallel(),
    )
    .add_system(exit_on_esc_system.system())
    .add_system(exit_on_window_close_system.system())
    .add_startup_system(setup.system())
    .add_system_to_stage(
        CoreStage::PostUpdate,
        camera_system::<SimpleOrthoProjection>.system(),
    )
    // display game stats
    .add_system(display_stats.system())
    // handle input
    .add_system(handle_keyboard_input.system().label(Label::Input))
    .add_system(handle_mouse_input.system().label(Label::Input))
    // handle AI
    .add_system(mob_ai.system().label(Label::Input))
    // handle movement + extra actions
    .add_system(player_move.system().after(Label::Input))
    .add_system(moving_object_update.system())
    .add_system(bomb_drop.system().after(Label::Input))
    // time effect update
    .add_system(perishable_tick.system().before(Label::Explosion))
    .add_system(immortality_tick.system())
    // handle bomb logic
    .add_system(handle_explosion.system().label(Label::Explosion))
    .add_system(
        fire_effect
            .system()
            .after(Label::Explosion)
            .before(Label::Burn),
    )
    .add_system(
        player_burn
            .system()
            .label(Label::Burn)
            .before(Label::Damage),
    )
    .add_system(bomb_burn.system().label(Label::Burn))
    .add_system(destructible_wall_burn.system().label(Label::Burn))
    .add_system(item_burn.system().label(Label::Burn))
    .add_system(exit_burn.system().label(Label::Burn))
    // player specifics
    .add_system(pick_up_item.system())
    .add_system(melee_attack.system().before(Label::Damage))
    .add_system(player_damage.system().label(Label::Damage))
    // animation
    .add_system(animate_fuse.system())
    .add_system(animate_immortality.system())
    // game end check
    .add_system_to_stage(Stage::GameEndCheck, finish_level.system())
    .add_system_to_stage(Stage::GameEndCheck, fail_level.system())
    .insert_resource(Level {
        sublevel: SubLevel::Regular(1),
        world: 1,
    })
    .insert_resource(GameScore(0))
    .add_event::<PlayerActionEvent>()
    .add_event::<ExplosionEvent>()
    .add_event::<BurnEvent>()
    .add_event::<DamageEvent>();

    app.run();
}
