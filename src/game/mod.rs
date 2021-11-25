use bevy::{ecs as bevy_ecs, prelude::*, render::camera::camera_system};

use crate::AppState;

use self::{camera::SimpleOrthoProjection, events::*, resources::*, systems::*};

pub mod camera;
pub mod components;
pub mod constants;
pub mod events;
pub mod resources;
pub mod systems;
pub mod types;
pub mod utils;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum Label {
    Setup,
    TimeUpdate,
    BombRestockEvent,
    ItemSpawn,
    Input,
    PlayerMovement,
    BombSpawn,
    FireSpawn,
    BurnEvent,
    PlayerSpawn,
    DamageEvent,
    DamageApplication,
    PlayerDeathEvent,
}

pub fn add_common_game_systems(app: &mut App, state: AppState) {
    app.add_system_set(
        SystemSet::on_enter(state).with_system(setup_penguin_portraits.exclusive_system()),
    )
    .add_system_set(
        SystemSet::on_update(state)
            // time effect update
            .with_system(
                move_cooldown_tick
                    .exclusive_system()
                    .label(Label::TimeUpdate),
            )
            .with_system(
                perishable_tick
                    .exclusive_system()
                    .label(Label::TimeUpdate)
                    .label(Label::ItemSpawn)
                    .label(Label::BombRestockEvent),
            )
            .with_system(immortality_tick.exclusive_system().label(Label::TimeUpdate))
            // handle input
            .with_system(handle_keyboard_input.exclusive_system().label(Label::Input))
            .with_system(handle_mouse_input.exclusive_system().label(Label::Input))
            // handle AI
            .with_system(mob_ai.exclusive_system().label(Label::Input))
            .with_system(
                bot_ai
                    .exclusive_system()
                    .label(Label::Input)
                    .after(Label::TimeUpdate),
            )
            // handle movement
            .with_system(
                player_move
                    .exclusive_system()
                    .label(Label::PlayerMovement)
                    .after(Label::Input)
                    .after(Label::TimeUpdate),
            )
            .with_system(
                moving_object_update
                    .exclusive_system()
                    .after(Label::TimeUpdate),
            )
            // handle bomb logic
            .with_system(
                bomb_drop
                    .exclusive_system()
                    .label(Label::BombSpawn)
                    .after(Label::Input),
            )
            .with_system(
                bomb_restock
                    .exclusive_system()
                    .after(Label::BombRestockEvent),
            )
            .with_system(
                handle_explosion
                    .exclusive_system()
                    .label(Label::FireSpawn)
                    .after(Label::TimeUpdate),
            )
            .with_system(
                fire_effect
                    .exclusive_system()
                    .label(Label::BurnEvent)
                    .after(Label::FireSpawn),
            )
            .with_system(
                player_burn
                    .exclusive_system()
                    .label(Label::DamageEvent)
                    .after(Label::BurnEvent)
                    .after(Label::PlayerMovement),
            )
            .with_system(
                bomb_burn
                    .exclusive_system()
                    .after(Label::BurnEvent)
                    .after(Label::BombSpawn),
            )
            .with_system(
                destructible_wall_burn
                    .exclusive_system()
                    .after(Label::BurnEvent),
            )
            .with_system(item_burn.exclusive_system().after(Label::BurnEvent))
            .with_system(
                exit_burn
                    .exclusive_system()
                    .label(Label::PlayerSpawn)
                    .after(Label::BurnEvent),
            )
            // player specifics
            .with_system(pick_up_item.exclusive_system().after(Label::ItemSpawn))
            .with_system(
                melee_attack
                    .exclusive_system()
                    .label(Label::DamageEvent)
                    .after(Label::PlayerSpawn)
                    .after(Label::PlayerMovement),
            )
            .with_system(
                player_damage
                    .exclusive_system()
                    .label(Label::DamageApplication)
                    .label(Label::PlayerDeathEvent)
                    .after(Label::PlayerMovement),
            )
            // animation
            .with_system(animate_fuse.exclusive_system().after(Label::TimeUpdate))
            .with_system(
                animate_immortality
                    .exclusive_system()
                    .after(Label::TimeUpdate),
            ),
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
            .add_event::<BombRestockEvent>()
            .add_event::<DamageEvent>()
            .add_event::<BurnEvent>()
            .add_event::<PlayerDeathEvent>()
            .add_system_to_stage(
                CoreStage::PostUpdate,
                camera_system::<SimpleOrthoProjection>,
            )
            .add_system_set(SystemSet::on_enter(AppState::Paused).with_system(hud_indicate_pause))
            .add_system_set(SystemSet::on_update(AppState::Paused).with_system(pop_state_on_enter));
    }
}
