use bevy::{ecs as bevy_ecs, prelude::*};

use crate::AppState;

use self::{events::*, resources::*, systems::*};

mod ai;
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
            .with_system(bomb_tick.exclusive_system().label(Label::TimeUpdate))
            .with_system(fire_tick.exclusive_system().label(Label::TimeUpdate))
            .with_system(
                crumbling_tick
                    .exclusive_system()
                    .label(Label::TimeUpdate)
                    .label(Label::ItemSpawn),
            )
            .with_system(
                burning_item_tick
                    .exclusive_system()
                    .label(Label::TimeUpdate),
            )
            .with_system(immortality_tick.exclusive_system().label(Label::TimeUpdate))
            // handle input
            .with_system(
                handle_user_input
                    .exclusive_system()
                    .label(Label::Input)
                    .after(crate::common::Label::InputMapping),
            )
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
                bomb_update
                    .exclusive_system()
                    .label(Label::BombRestockEvent)
                    .label(Label::FireSpawn)
                    .after(Label::TimeUpdate),
            )
            .with_system(
                bomb_restock
                    .exclusive_system()
                    .after(Label::BombRestockEvent),
            )
            .with_system(
                fire_effect
                    .exclusive_system()
                    .label(Label::BurnEvent)
                    .after(Label::TimeUpdate)
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
        app.init_resource::<HUDColors>()
            .init_resource::<GameTextures>()
            .init_resource::<Sounds>()
            .add_event::<PlayerActionEvent>()
            .add_event::<ExplosionEvent>()
            .add_event::<BombRestockEvent>()
            .add_event::<DamageEvent>()
            .add_event::<BurnEvent>()
            .add_event::<PlayerDeathEvent>()
            .add_system_set(SystemSet::on_enter(AppState::Paused).with_system(hud_indicate_pause))
            .add_system_set(
                // these 2 do not need to be marked as .after(InputMapping) since they
                // are regular systems that already run after exclusive ones
                SystemSet::on_update(AppState::Paused)
                    .with_system(pop_state_on_enter)
                    .with_system(pop_state_fallthrough_on_esc),
            );
    }
}
