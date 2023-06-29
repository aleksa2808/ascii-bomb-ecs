use bevy::prelude::*;

use crate::{
    game::{
        components::*,
        constants::{TILE_HEIGHT, TILE_WIDTH},
        resources::*,
        types::{BotDifficulty, Cooldown},
        utils::{get_x, get_y},
    },
    AppState,
};

use super::{
    constants::ROUND_START_FREEZE_SECS,
    resources::{BattleModeContext, BattleModeState, FreezeTimer},
    types::PenguinControlType,
};

pub fn spawn_battle_mode_players(
    commands: &mut Commands,
    game_textures: &GameTextures,
    map_size: MapSize,
    players: &[(Penguin, PenguinControlType)],
    bot_difficulty: BotDifficulty,
) -> Vec<Position> {
    let possible_player_spawn_positions = [
        (1, 1),
        (map_size.rows - 2, map_size.columns - 2),
        (1, map_size.columns - 2),
        (map_size.rows - 2, 1),
        (3, 5),
        (map_size.rows - 4, map_size.columns - 6),
        (3, map_size.columns - 6),
        (map_size.rows - 4, 5),
    ];
    let mut possible_player_spawn_positions =
        possible_player_spawn_positions
            .iter()
            .map(|(y, x)| Position {
                y: *y as isize,
                x: *x as isize,
            });

    let mut player_spawn_positions = vec![];

    let mut spawn_player = |penguin_tag: Penguin, penguin_control_type: PenguinControlType| {
        let player_spawn_position = possible_player_spawn_positions.next().unwrap();
        let base_texture = game_textures.get_penguin_texture(penguin_tag).clone();
        let immortal_texture = game_textures.immortal_penguin.clone();
        let mut entity_commands = commands.spawn((
            SpriteBundle {
                texture: base_texture.clone(),
                transform: Transform::from_xyz(
                    get_x(player_spawn_position.x),
                    get_y(player_spawn_position.y),
                    50.0,
                ),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                    ..Default::default()
                },
                ..Default::default()
            },
            BaseTexture(base_texture),
            ImmortalTexture(immortal_texture),
            Player,
            penguin_tag,
            Health {
                lives: 1,
                max_health: 1,
                health: 1,
            },
            player_spawn_position,
            SpawnPosition(player_spawn_position),
            BombSatchel {
                bombs_available: 1,
                bomb_range: 2,
            },
            TeamID(penguin_tag.0),
        ));
        match penguin_control_type {
            PenguinControlType::Human(i) => {
                entity_commands.insert(HumanControlled(i));
            }
            PenguinControlType::Bot => {
                entity_commands.insert((
                    BotAI {
                        difficulty: bot_difficulty,
                    },
                    MoveCooldown(Cooldown::from_seconds(match bot_difficulty {
                        BotDifficulty::Easy => 0.3,
                        BotDifficulty::Medium => 0.25,
                        BotDifficulty::Hard => 0.2,
                    })),
                ));
            }
        }

        player_spawn_positions.push(player_spawn_position);
    };

    for (penguin_tag, penguin_control_type) in players {
        spawn_player(*penguin_tag, *penguin_control_type);
    }

    player_spawn_positions
}

pub fn get_battle_mode_map_size_fill(player_count: usize) -> (MapSize, f32) {
    if player_count > 4 {
        (
            MapSize {
                rows: 13,
                columns: 17,
            },
            70.0,
        )
    } else {
        (
            MapSize {
                rows: 11,
                columns: 15,
            },
            60.0,
        )
    }
}

pub fn start_round(
    mut battle_mode_context: ResMut<BattleModeContext>,
    mut commands: Commands,
    mut state: ResMut<State<AppState>>,
) {
    battle_mode_context.state = BattleModeState::InGame;
    commands.insert_resource(FreezeTimer(Timer::from_seconds(
        ROUND_START_FREEZE_SECS,
        TimerMode::Once,
    )));
    state.push(AppState::RoundStartFreeze).unwrap();
}
