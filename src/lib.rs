mod camera;
mod components;
mod constants;
mod events;
mod helpers;
mod item;
mod resources;
mod systems;
mod types;

use bevy::{input::system::exit_on_esc_system, prelude::*, window::exit_on_window_close_system};
use wasm_bindgen::prelude::*;

use crate::camera::SimpleOrthoProjection;
use crate::events::*;
use crate::systems::*;

#[wasm_bindgen]
pub fn run() {
    let mut app = App::build();
    app.insert_resource(WindowDescriptor {
        title: "ascii-bomb-ecs".to_string(),
        height: 880.0,
        width: 900.0,
        ..Default::default()
    })
    .add_plugins(DefaultPlugins);

    // when building for Web, use WebGL2 rendering
    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);

    use bevy::render::camera::camera_system;

    app.add_system(exit_on_esc_system.system())
        .add_system(exit_on_window_close_system.system())
        .add_startup_system(setup.system())
        .add_system_to_stage(
            CoreStage::PostUpdate,
            camera_system::<SimpleOrthoProjection>.system(),
        )
        // handle input
        .add_system(handle_keyboard_input.system().label("input"))
        .add_system(handle_mouse_input.system().label("input"))
        // handle movement + extra actions
        .add_system(player_move.system().after("input"))
        .add_system(moving_object_update.system())
        .add_system(bomb_drop.system().after("input"))
        // time effect update
        .add_system(perishable_tick.system().before("explosion"))
        .add_system(immortality_tick.system())
        // handle bomb logic
        .add_system(handle_explosion.system().label("explosion"))
        .add_system(fire_effect.system().after("explosion").before("burn"))
        .add_system(player_burn.system().label("burn").before("damage"))
        .add_system(bomb_burn.system().label("burn"))
        .add_system(destructible_wall_burn.system().label("burn"))
        .add_system(item_burn.system().label("burn"))
        // player specifics
        .add_system(pick_up_item.system())
        .add_system(melee_attack.system().before("damage"))
        .add_system(player_damage.system().label("damage"))
        // animation
        .add_system(animate_fuse.system())
        .add_system(animate_immortality.system())
        .add_event::<PlayerActionEvent>()
        .add_event::<ExplosionEvent>()
        .add_event::<BurnEvent>()
        .add_event::<DamageEvent>();

    app.run();
}
