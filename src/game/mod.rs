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

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum Set {
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
    app.add_system(
        // time effect update
        move_cooldown_tick
            .in_set(Set::TimeUpdate)
            .in_set(OnUpdate(state)),
    )
    .add_systems(
        (bomb_tick, apply_system_buffers)
            .chain()
            .in_set(Set::TimeUpdate)
            .in_set(OnUpdate(state)),
    )
    .add_systems(
        (fire_tick, apply_system_buffers)
            .chain()
            .in_set(Set::TimeUpdate)
            .in_set(OnUpdate(state)),
    )
    .add_systems(
        (crumbling_tick, apply_system_buffers)
            .chain()
            .in_set(Set::TimeUpdate)
            .in_set(Set::ItemSpawn)
            .in_set(OnUpdate(state)),
    )
    .add_systems(
        (burning_item_tick, apply_system_buffers)
            .chain()
            .in_set(Set::TimeUpdate)
            .in_set(OnUpdate(state)),
    )
    .add_systems(
        (immortality_tick, apply_system_buffers)
            .chain()
            .in_set(Set::TimeUpdate)
            .in_set(OnUpdate(state)),
    )
    // handle input
    .add_systems(
        (handle_user_input, apply_system_buffers)
            .chain()
            .in_set(Set::Input)
            .after(crate::common::Label::InputMapping)
            .in_set(OnUpdate(state)),
    )
    // handle AI
    .add_systems(
        (mob_ai, apply_system_buffers)
            .chain()
            .in_set(Set::Input)
            .in_set(OnUpdate(state)),
    )
    .add_systems(
        (bot_ai, apply_system_buffers)
            .chain()
            .in_set(Set::Input)
            .after(Set::TimeUpdate)
            .in_set(OnUpdate(state)),
    )
    // handle movement
    .add_systems(
        (player_move, apply_system_buffers)
            .chain()
            .in_set(Set::PlayerMovement)
            .after(Set::Input)
            .after(Set::TimeUpdate)
            .in_set(OnUpdate(state)),
    )
    .add_systems(
        (moving_object_update, apply_system_buffers)
            .chain()
            .after(Set::TimeUpdate)
            .in_set(OnUpdate(state)),
    )
    // handle bomb logic
    .add_systems(
        (bomb_drop, apply_system_buffers)
            .chain()
            .in_set(Set::BombSpawn)
            .after(Set::Input)
            .in_set(OnUpdate(state)),
    )
    .add_systems(
        (bomb_update, apply_system_buffers)
            .chain()
            .in_set(Set::BombRestockEvent)
            .in_set(Set::FireSpawn)
            .after(Set::TimeUpdate)
            .in_set(OnUpdate(state)),
    )
    .add_systems(
        (bomb_restock, apply_system_buffers)
            .chain()
            .after(Set::BombRestockEvent)
            .in_set(OnUpdate(state)),
    )
    .add_systems(
        (fire_effect, apply_system_buffers)
            .chain()
            .in_set(Set::BurnEvent)
            .after(Set::TimeUpdate)
            .after(Set::FireSpawn)
            .in_set(OnUpdate(state)),
    )
    .add_systems(
        (player_burn, apply_system_buffers)
            .chain()
            .in_set(Set::DamageEvent)
            .after(Set::BurnEvent)
            .after(Set::PlayerMovement)
            .in_set(OnUpdate(state)),
    )
    .add_systems(
        (bomb_burn, apply_system_buffers)
            .chain()
            .after(Set::BurnEvent)
            .after(Set::BombSpawn)
            .in_set(OnUpdate(state)),
    )
    .add_systems(
        (destructible_wall_burn, apply_system_buffers)
            .chain()
            .after(Set::BurnEvent)
            .in_set(OnUpdate(state)),
    )
    .add_systems(
        (item_burn, apply_system_buffers)
            .chain()
            .after(Set::BurnEvent)
            .in_set(OnUpdate(state)),
    )
    .add_systems(
        (exit_burn, apply_system_buffers)
            .chain()
            .in_set(Set::PlayerSpawn)
            .after(Set::BurnEvent)
            .in_set(OnUpdate(state)),
    )
    // player specifics
    .add_systems(
        (pick_up_item, apply_system_buffers)
            .chain()
            .after(Set::ItemSpawn)
            .in_set(OnUpdate(state)),
    )
    .add_systems(
        (melee_attack, apply_system_buffers)
            .chain()
            .in_set(Set::DamageEvent)
            .after(Set::PlayerSpawn)
            .after(Set::PlayerMovement)
            .in_set(OnUpdate(state)),
    )
    .add_systems(
        (player_damage, apply_system_buffers)
            .chain()
            .in_set(Set::DamageApplication)
            .in_set(Set::PlayerDeathEvent)
            .after(Set::PlayerMovement)
            .in_set(OnUpdate(state)),
    )
    // animation
    .add_systems(
        (animate_fuse, apply_system_buffers)
            .chain()
            .after(Set::TimeUpdate)
            .in_set(OnUpdate(state)),
    )
    .add_systems(
        (animate_immortality, apply_system_buffers)
            .chain()
            .after(Set::TimeUpdate)
            .in_set(OnUpdate(state)),
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
            .add_system(hud_indicate_pause.in_schedule(OnEnter(AppState::Paused)))
            .add_systems(
                (pop_state_on_enter, pop_state_fallthrough_on_esc)
                    .in_set(OnUpdate(AppState::Paused))
                    .after(crate::common::Label::InputMapping),
            )
            .add_system(pause_teardown.in_schedule(OnExit(AppState::Paused)));
    }
}
