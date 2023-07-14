use bevy::{
    ecs::{self as bevy_ecs, schedule::SystemConfigs},
    prelude::*,
};

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
    TimeUpdate,
    BombRestockEvent,
    ItemSpawn,
    Input,
    PlayerMovement,
    BombSpawn,
    FireSpawn,
    PlayerSpawn,
    DamageEvent,
    DamageApplication,
    PlayerDeathEvent,
}

pub fn common_game_systems() -> SystemConfigs {
    (
        // time effect update
        (
            move_cooldown_tick,
            bomb_tick,
            (fire_tick, apply_deferred),
            (crumbling_tick, apply_deferred)
                .chain()
                .in_set(Set::ItemSpawn),
            (burning_item_tick, apply_deferred).chain(),
            (immortality_tick, apply_deferred).chain(),
        )
            .in_set(Set::TimeUpdate),
        (
            // handle user input
            (handle_user_input, apply_deferred)
                .chain()
                .after(crate::common::Label::InputMapping),
            // handle AI input
            mob_ai,
            bot_ai.after(Set::TimeUpdate),
        )
            .in_set(Set::Input),
        // handle movement
        (
            (player_move, apply_deferred)
                .chain()
                .in_set(Set::PlayerMovement)
                .after(Set::Input),
            (moving_object_update, apply_deferred).chain(),
        )
            .after(Set::TimeUpdate),
        // handle bomb logic
        (bomb_drop, apply_deferred)
            .chain()
            .in_set(Set::BombSpawn)
            .after(Set::Input),
        (bomb_update, apply_deferred)
            .chain()
            .in_set(Set::BombRestockEvent)
            .in_set(Set::FireSpawn)
            .after(Set::TimeUpdate),
        bomb_restock.after(Set::BombRestockEvent),
        (
            // burn
            fire_effect.after(Set::TimeUpdate).after(Set::FireSpawn),
            // burn reactions
            (
                player_burn
                    .in_set(Set::DamageEvent)
                    .after(Set::PlayerMovement),
                bomb_burn.after(Set::BombSpawn),
                (destructible_wall_burn, apply_deferred).chain(),
                (item_burn, apply_deferred).chain(),
                (exit_burn, apply_deferred).chain().in_set(Set::PlayerSpawn),
            ),
        )
            .chain(),
        // player specifics
        (pick_up_item, apply_deferred).chain().after(Set::ItemSpawn),
        melee_attack
            .in_set(Set::DamageEvent)
            .after(Set::PlayerSpawn)
            .after(Set::PlayerMovement),
        (player_damage, apply_deferred)
            .chain()
            .in_set(Set::DamageApplication)
            .in_set(Set::PlayerDeathEvent)
            .after(Set::PlayerMovement)
            .after(Set::DamageEvent),
        // animation
        (animate_fuse, animate_immortality).after(Set::TimeUpdate),
    )
        .into_configs()
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
                (unpause_on_enter, quit_game_on_esc)
                    .after(crate::common::Label::InputMapping)
                    .run_if(in_state(AppState::Paused)),
            )
            .add_systems(OnExit(AppState::Paused), pause_teardown);
    }
}
