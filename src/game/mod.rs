use bevy::{
    ecs::{self as bevy_ecs, schedule::ScheduleConfigs, system::ScheduleSystem},
    prelude::*,
};

use crate::AppState;

use self::{messages::*, resources::*, systems::*};

mod ai;
pub mod components;
pub mod constants;
pub mod messages;
pub mod resources;
pub mod systems;
pub mod types;
pub mod utils;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum Set {
    TimeUpdate,
    BombRestockMessage,
    ItemSpawn,
    ItemDespawn,
    Input,
    PlayerMovement,
    MovingObjectUpdate,
    BombSpawn,
    FireSpawn,
    PlayerSpawn,
    DamageMessage,
    DamageApplication,
    PlayerDeathMessage,
}

pub fn common_game_systems() -> ScheduleConfigs<ScheduleSystem> {
    (
        // time effect update
        (
            move_cooldown_tick,
            bomb_tick,
            (fire_tick, burning_item_tick, immortality_tick),
            crumbling_tick.in_set(Set::ItemSpawn),
        )
            .in_set(Set::TimeUpdate),
        (
            // handle user input
            handle_user_input.after(crate::common::Label::InputMapping),
            // handle AI input
            mob_ai,
            bot_ai.after(Set::TimeUpdate),
        )
            .in_set(Set::Input),
        // handle movement
        (
            player_move.in_set(Set::PlayerMovement).after(Set::Input),
            moving_object_update.in_set(Set::MovingObjectUpdate),
        )
            .after(Set::TimeUpdate),
        // handle bomb logic
        bomb_drop.in_set(Set::BombSpawn).after(Set::Input),
        explode_bombs
            .in_set(Set::BombRestockMessage)
            .in_set(Set::FireSpawn)
            .after(Set::TimeUpdate)
            .after(Set::MovingObjectUpdate),
        bomb_restock.after(Set::BombRestockMessage),
        (
            // burn
            fire_effect.after(Set::TimeUpdate).after(Set::FireSpawn),
            // burn reactions
            (
                player_burn
                    .in_set(Set::DamageMessage)
                    .after(Set::PlayerMovement),
                bomb_burn
                    .after(Set::BombSpawn)
                    .after(Set::MovingObjectUpdate),
                destructible_wall_burn,
                item_burn.in_set(Set::ItemDespawn),
                exit_burn.in_set(Set::PlayerSpawn),
            ),
        )
            .chain(),
        // player specifics
        melee_attack
            .in_set(Set::DamageMessage)
            .after(Set::PlayerSpawn)
            .after(Set::PlayerMovement),
        player_damage
            .in_set(Set::DamageApplication)
            .in_set(Set::PlayerDeathMessage)
            .after(Set::PlayerMovement)
            .after(Set::DamageMessage),
        pick_up_item
            .after(Set::PlayerDeathMessage)
            .after(Set::ItemSpawn)
            .after(Set::ItemDespawn),
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
            .add_message::<PlayerActionMessage>()
            .add_message::<ExplosionMessage>()
            .add_message::<BombRestockMessage>()
            .add_message::<DamageMessage>()
            .add_message::<BurnMessage>()
            .add_message::<PlayerDeathMessage>()
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
