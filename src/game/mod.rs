use bevy::{prelude::*, render::camera::camera_system};

use crate::{AppState, Label};

use self::{camera::SimpleOrthoProjection, events::*, resources::*, systems::*};

pub mod camera;
pub mod components;
pub mod constants;
pub mod events;
pub mod resources;
pub mod systems;
pub mod types;
pub mod utils;

pub fn add_common_game_systems(app: &mut App, state: AppState) {
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

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HUDMaterials>()
            .init_resource::<Textures>()
            .init_resource::<Sounds>()
            .add_event::<PlayerActionEvent>()
            .add_event::<ExplosionEvent>()
            .add_event::<BurnEvent>()
            .add_event::<DamageEvent>()
            .add_system_to_stage(
                CoreStage::PostUpdate,
                camera_system::<SimpleOrthoProjection>,
            )
            .add_system_set(SystemSet::on_enter(AppState::Paused).with_system(hud_indicate_pause))
            .add_system_set(SystemSet::on_update(AppState::Paused).with_system(pop_state_on_enter));
    }
}
