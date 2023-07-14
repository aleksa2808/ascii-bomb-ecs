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
    app.add_systems(
        Update,
        // time effect update
        move_cooldown_tick
            .in_set(Set::TimeUpdate)
            .run_if(in_state(state)),
    )
    .add_systems(
        Update,
        (bomb_tick, apply_deferred)
            .chain()
            .in_set(Set::TimeUpdate)
            .run_if(in_state(state)),
    )
    .add_systems(
        Update,
        (fire_tick, apply_deferred)
            .chain()
            .in_set(Set::TimeUpdate)
            .run_if(in_state(state)),
    )
    .add_systems(
        Update,
        (crumbling_tick, apply_deferred)
            .chain()
            .in_set(Set::TimeUpdate)
            .in_set(Set::ItemSpawn)
            .run_if(in_state(state)),
    )
    .add_systems(
        Update,
        (burning_item_tick, apply_deferred)
            .chain()
            .in_set(Set::TimeUpdate)
            .run_if(in_state(state)),
    )
    .add_systems(
        Update,
        (immortality_tick, apply_deferred)
            .chain()
            .in_set(Set::TimeUpdate)
            .run_if(in_state(state)),
    )
    // handle input
    .add_systems(
        Update,
        (handle_user_input, apply_deferred)
            .chain()
            .in_set(Set::Input)
            .after(crate::common::Label::InputMapping)
            .run_if(in_state(state)),
    )
    // handle AI
    .add_systems(
        Update,
        (mob_ai, apply_deferred)
            .chain()
            .in_set(Set::Input)
            .run_if(in_state(state)),
    )
    .add_systems(
        Update,
        (bot_ai, apply_deferred)
            .chain()
            .in_set(Set::Input)
            .after(Set::TimeUpdate)
            .run_if(in_state(state)),
    )
    // handle movement
    .add_systems(
        Update,
        (player_move, apply_deferred)
            .chain()
            .in_set(Set::PlayerMovement)
            .after(Set::Input)
            .after(Set::TimeUpdate)
            .run_if(in_state(state)),
    )
    .add_systems(
        Update,
        (moving_object_update, apply_deferred)
            .chain()
            .after(Set::TimeUpdate)
            .run_if(in_state(state)),
    )
    // handle bomb logic
    .add_systems(
        Update,
        (bomb_drop, apply_deferred)
            .chain()
            .in_set(Set::BombSpawn)
            .after(Set::Input)
            .run_if(in_state(state)),
    )
    .add_systems(
        Update,
        (bomb_update, apply_deferred)
            .chain()
            .in_set(Set::BombRestockEvent)
            .in_set(Set::FireSpawn)
            .after(Set::TimeUpdate)
            .run_if(in_state(state)),
    )
    .add_systems(
        Update,
        (bomb_restock, apply_deferred)
            .chain()
            .after(Set::BombRestockEvent)
            .run_if(in_state(state)),
    )
    .add_systems(
        Update,
        (fire_effect, apply_deferred)
            .chain()
            .in_set(Set::BurnEvent)
            .after(Set::TimeUpdate)
            .after(Set::FireSpawn)
            .run_if(in_state(state)),
    )
    .add_systems(
        Update,
        (player_burn, apply_deferred)
            .chain()
            .in_set(Set::DamageEvent)
            .after(Set::BurnEvent)
            .after(Set::PlayerMovement)
            .run_if(in_state(state)),
    )
    .add_systems(
        Update,
        (bomb_burn, apply_deferred)
            .chain()
            .after(Set::BurnEvent)
            .after(Set::BombSpawn)
            .run_if(in_state(state)),
    )
    .add_systems(
        Update,
        (destructible_wall_burn, apply_deferred)
            .chain()
            .after(Set::BurnEvent)
            .run_if(in_state(state)),
    )
    .add_systems(
        Update,
        (item_burn, apply_deferred)
            .chain()
            .after(Set::BurnEvent)
            .run_if(in_state(state)),
    )
    .add_systems(
        Update,
        (exit_burn, apply_deferred)
            .chain()
            .in_set(Set::PlayerSpawn)
            .after(Set::BurnEvent)
            .run_if(in_state(state)),
    )
    // player specifics
    .add_systems(
        Update,
        (pick_up_item, apply_deferred)
            .chain()
            .after(Set::ItemSpawn)
            .run_if(in_state(state)),
    )
    .add_systems(
        Update,
        (melee_attack, apply_deferred)
            .chain()
            .in_set(Set::DamageEvent)
            .after(Set::PlayerSpawn)
            .after(Set::PlayerMovement)
            .run_if(in_state(state)),
    )
    .add_systems(
        Update,
        (player_damage, apply_deferred)
            .chain()
            .in_set(Set::DamageApplication)
            .in_set(Set::PlayerDeathEvent)
            .after(Set::PlayerMovement)
            .run_if(in_state(state)),
    )
    // animation
    .add_systems(
        Update,
        (animate_fuse, apply_deferred)
            .chain()
            .after(Set::TimeUpdate)
            .run_if(in_state(state)),
    )
    .add_systems(
        Update,
        (animate_immortality, apply_deferred)
            .chain()
            .after(Set::TimeUpdate)
            .run_if(in_state(state)),
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
            .add_systems(OnEnter(AppState::Paused), hud_indicate_pause)
            .add_systems(
                Update,
                (pop_state_on_enter, pop_state_fallthrough_on_esc)
                    .run_if(in_state(AppState::Paused))
                    .after(crate::common::Label::InputMapping),
            )
            .add_systems(OnExit(AppState::Paused), pause_teardown);
    }
}
