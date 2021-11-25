use std::collections::HashSet;

use bevy::{app::Events, prelude::*, render::camera::Camera};
use rand::{prelude::IteratorRandom, Rng};

use crate::{
    common::{
        constants::{COLORS, PIXEL_SCALE},
        resources::*,
    },
    game::{
        components::*,
        constants::{TILE_HEIGHT, TILE_WIDTH},
        events::*,
        resources::*,
        types::{Direction, *},
        utils::{format_hud_time, generate_item_at_position, get_x, get_y, init_hud, spawn_map},
    },
    map_transition::MapTransitionInput,
    AppState,
};

use super::{
    constants::{BATTLE_MODE_ROUND_DURATION_SECS, ROUND_START_FREEZE_SECS},
    resources::*,
    utils::*,
};

pub fn setup_battle_mode(
    mut commands: Commands,
    mut textures: ResMut<Textures>,
    fonts: Res<Fonts>,
    base_color_materials: Res<BaseColorMaterials>,
    hud_materials: Res<HUDMaterials>,
    battle_mode_configuration: Res<BattleModeConfiguration>,
) {
    let world_id = WorldID(rand::thread_rng().gen_range(1..=3));
    textures.set_map_textures(world_id);

    let (map_size, percent_of_passable_positions_to_fill) = get_battle_mode_map_size_fill(
        battle_mode_configuration.amount_of_players + battle_mode_configuration.amount_of_bots,
    );

    // spawn HUD
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                ..Default::default()
            },
            material: base_color_materials.none.clone(),
            ..Default::default()
        })
        .insert(UIRoot)
        .insert(UIComponent)
        .with_children(|parent| {
            init_hud(
                parent,
                &hud_materials,
                &fonts,
                (map_size.columns * TILE_WIDTH) as f32,
                world_id,
                true,
                true,
                None,
            );
        });

    let players: Vec<(Penguin, PenguinControlType)> =
        (0..battle_mode_configuration.amount_of_players + battle_mode_configuration.amount_of_bots)
            .into_iter()
            .map(|i| {
                (
                    Penguin(i),
                    if i < battle_mode_configuration.amount_of_players {
                        PenguinControlType::Human(i)
                    } else {
                        PenguinControlType::Bot
                    },
                )
            })
            .collect();

    let leaderboard = Leaderboard {
        scores: players.iter().map(|(pt, _)| (*pt, 0)).collect(),
        winning_score: battle_mode_configuration.winning_score,
    };
    commands.insert_resource(BattleModeContext {
        state: BattleModeState::RoundSetup,
        players,
        leaderboard,
        round_outcome: None,
        percent_of_passable_positions_to_fill,
    });
    commands.insert_resource(GameContext {
        pausable: battle_mode_configuration.amount_of_players > 0,
        reduced_loot: true,
    });
    commands.insert_resource(world_id);
    commands.insert_resource(map_size);

    commands.remove_resource::<BattleModeConfiguration>();
}

pub fn battle_mode_manager(
    mut commands: Commands,
    textures: Res<Textures>,
    map_size: Res<MapSize>,
    mut battle_mode_context: ResMut<BattleModeContext>,
    game_option_store: Res<GameOptionStore>,
    mut state: ResMut<State<AppState>>,
    query: Query<Entity, (Without<Camera>, Without<UIComponent>)>,
    query2: Query<Entity, With<PenguinPortrait>>,
    mut query4: Query<&mut Text, With<GameTimerDisplay>>,
) {
    loop {
        match battle_mode_context.state {
            BattleModeState::RoundSetup => {
                // map generation //
                let player_spawn_positions = spawn_battle_mode_players(
                    &mut commands,
                    &textures,
                    *map_size,
                    &battle_mode_context.players,
                );

                let wall_entity_reveal_groups = spawn_map(
                    &mut commands,
                    &textures,
                    *map_size,
                    battle_mode_context.percent_of_passable_positions_to_fill,
                    true,
                    &player_spawn_positions,
                    &[],
                    false,
                );

                commands.insert_resource(GameTimer(Timer::from_seconds(
                    BATTLE_MODE_ROUND_DURATION_SECS as f32,
                    false,
                )));
                // update HUD clock
                query4.single_mut().sections[0].value =
                    format_hud_time(BATTLE_MODE_ROUND_DURATION_SECS);
                commands.insert_resource(WallOfDeath::Dormant(Timer::from_seconds(
                    BATTLE_MODE_ROUND_DURATION_SECS as f32 / 2.0,
                    false,
                )));

                battle_mode_context.round_outcome = None;

                if game_option_store.get(GameOption::Transition) {
                    battle_mode_context.state = BattleModeState::MapTransition;
                    commands.insert_resource(MapTransitionInput {
                        wall_entity_reveal_groups,
                    });
                    state.push(AppState::MapTransition).unwrap();
                } else {
                    battle_mode_context.state = BattleModeState::InGame;
                    state.push(AppState::BattleModeInGame).unwrap();
                }
            }
            BattleModeState::MapTransition => {
                battle_mode_context.state = BattleModeState::InGame;
                state.push(AppState::BattleModeInGame).unwrap();
            }
            BattleModeState::InGame => {
                match battle_mode_context.round_outcome {
                    Some(result) => {
                        match result {
                            RoundOutcome::Tie => {
                                println!("Round over with no winners!");
                            }
                            RoundOutcome::Winner(penguin) => {
                                println!("Player {:?} won the round!", penguin.0);
                                *battle_mode_context
                                    .leaderboard
                                    .scores
                                    .get_mut(&penguin)
                                    .unwrap() += 1;
                            }
                        }

                        for entity in query.iter() {
                            commands.entity(entity).despawn_recursive();
                        }

                        // clear penguin portraits
                        for entity in query2.iter() {
                            commands.entity(entity).despawn_recursive();
                        }

                        battle_mode_context.state = BattleModeState::LeaderboardDisplay;
                        state.push(AppState::LeaderboardDisplay).unwrap();
                    }
                    None => {
                        // abrupt exit
                        state.replace(AppState::MainMenu).unwrap();
                    }
                }
            }
            BattleModeState::LeaderboardDisplay => {
                if let Some((penguin, _)) = battle_mode_context
                    .leaderboard
                    .scores
                    .iter()
                    .find(|(_, s)| **s == battle_mode_context.leaderboard.winning_score)
                {
                    println!("Tournament complete! Winner: {:?}", penguin.0);
                    state.replace(AppState::MainMenu).unwrap();
                } else {
                    battle_mode_context.state = BattleModeState::RoundSetup;
                    continue;
                }
            }
        }
        break;
    }
}

pub fn trigger_round_start_freeze(mut commands: Commands, mut state: ResMut<State<AppState>>) {
    commands.insert_resource(FreezeTimer(Timer::from_seconds(
        ROUND_START_FREEZE_SECS,
        false,
    )));
    state.push(AppState::RoundStartFreeze).unwrap();
}

pub fn finish_freeze(
    mut commands: Commands,
    time: Res<Time>,
    mut freeze_timer: ResMut<FreezeTimer>,
    mut state: ResMut<State<AppState>>,
) {
    freeze_timer.0.tick(time.delta());
    if freeze_timer.0.finished() {
        commands.remove_resource::<FreezeTimer>();
        state.pop().unwrap();
    }
}

pub fn wall_of_death_update(
    mut commands: Commands,
    time: Res<Time>,
    textures: Res<Textures>,
    mut wall_of_death: ResMut<WallOfDeath>,
    map_size: Res<MapSize>,
    query: Query<&Position, (With<Wall>, Without<Destructible>)>,
    query2: Query<(Entity, &Position, Option<&Bomb>, Option<&Player>)>,
    mut ev_player_death_event: EventWriter<PlayerDeathEvent>,
    mut ev_bomb_restock: EventWriter<BombRestockEvent>,
) {
    let get_next_position_direction = |mut position: Position,
                                       mut direction: Direction|
     -> Option<(Position, Direction)> {
        let end_position = Position {
            y: map_size.rows as isize - 3,
            x: 3,
        };

        let walls: HashSet<Position> = query.iter().copied().collect();
        loop {
            if position == end_position {
                break None;
            }

            match position {
                Position { y: 1, x: 1 } | Position { y: 2, x: 2 } => {
                    direction = Direction::Right;
                }
                Position { y: 1, x } if x == map_size.columns as isize - 2 => {
                    direction = Direction::Down;
                }
                Position { y, x }
                    if y == map_size.rows as isize - 2 && x == map_size.columns as isize - 2 =>
                {
                    direction = Direction::Left;
                }
                Position { y, x: 2 } if y == map_size.rows as isize - 2 => {
                    direction = Direction::Up;
                }
                Position { y: 2, x } if x == map_size.columns as isize - 3 => {
                    direction = Direction::Down;
                }
                Position { y, x }
                    if y == map_size.rows as isize - 3 && x == map_size.columns as isize - 3 =>
                {
                    direction = Direction::Left;
                }
                _ => (),
            }

            position = position.offset(direction, 1);
            if !walls.contains(&position) {
                break Some((position, direction));
            }
        }
    };

    let mut clear_position_and_spawn_wall = |position: Position| {
        for (e, _, bomb, player) in query2.iter().filter(|(_, p, _, _)| **p == position) {
            commands.entity(e).despawn_recursive();

            if player.is_some() {
                println!("player died from wall of death: {:?}", e);
                ev_player_death_event.send(PlayerDeathEvent);
            }

            if let Some(bomb) = bomb {
                if let Some(owner) = bomb.owner {
                    ev_bomb_restock.send(BombRestockEvent {
                        satchel_owner: owner,
                    })
                }
            }
        }

        commands
            .spawn_bundle(SpriteBundle {
                material: textures.get_map_textures().wall.clone(),
                transform: Transform::from_xyz(get_x(position.x), get_y(position.y), 0.0),
                sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                ..Default::default()
            })
            .insert(Wall)
            .insert(Solid)
            .insert(position);
    };

    loop {
        let new_state = match *wall_of_death {
            WallOfDeath::Dormant(ref mut timer) => {
                timer.tick(time.delta());

                if timer.finished() {
                    println!("Wall of Death activated!");

                    Some(WallOfDeath::Active(ActiveWallOfDeath {
                        position: Position {
                            y: map_size.rows as isize - 1,
                            x: 1,
                        },
                        direction: Direction::Up,
                        cooldown: Cooldown::from_seconds(0.2),
                    }))
                } else {
                    None
                }
            }
            WallOfDeath::Active(ref mut active_wall_of_death) => {
                active_wall_of_death.cooldown.tick(time.delta());
                if active_wall_of_death.cooldown.ready() {
                    if let Some((position, direction)) = get_next_position_direction(
                        active_wall_of_death.position,
                        active_wall_of_death.direction,
                    ) {
                        active_wall_of_death.cooldown.trigger();
                        active_wall_of_death.position = position;
                        active_wall_of_death.direction = direction;

                        clear_position_and_spawn_wall(active_wall_of_death.position);

                        None
                    } else {
                        Some(WallOfDeath::Done)
                    }
                } else {
                    None
                }
            }
            WallOfDeath::Done => None,
        };

        if let Some(new_state) = new_state {
            *wall_of_death = new_state;
        } else {
            break;
        }
    }
}

pub fn on_death_item_pinata(
    mut commands: Commands,
    textures: Res<Textures>,
    map_size: Res<MapSize>,
    game_context: Res<GameContext>,
    query: Query<
        &Position,
        Or<(
            With<Player>,
            With<Solid>,
            With<Fire>,
            With<BurningItem>,
            With<Item>,
            With<Exit>,
        )>,
    >,
    mut ev_player_death_event: EventReader<PlayerDeathEvent>,
) {
    for _ in ev_player_death_event.iter() {
        let invalid_positions: HashSet<Position> = query.iter().copied().collect();
        let valid_positions = (0..map_size.rows)
            .map(|y| {
                (0..map_size.columns).map(move |x| Position {
                    y: y as isize,
                    x: x as isize,
                })
            })
            .flatten()
            .filter(|p| !invalid_positions.contains(p));
        for position in valid_positions.choose_multiple(&mut rand::thread_rng(), 3) {
            generate_item_at_position(
                position,
                &mut commands,
                &textures,
                game_context.reduced_loot,
            );
        }
    }
}

pub fn finish_round(
    game_timer: Res<GameTimer>,
    keyboard_input: Res<Input<KeyCode>>,
    mut battle_mode_context: ResMut<BattleModeContext>,
    query: Query<&Penguin, With<Player>>,
    mut state: ResMut<State<AppState>>,
) {
    let mut round_over = false;
    if game_timer.0.finished() || query.iter().count() == 0 {
        battle_mode_context.round_outcome = Some(RoundOutcome::Tie);
        round_over = true;
    } else if let Ok(penguin) = query.get_single() {
        battle_mode_context.round_outcome = Some(RoundOutcome::Winner(*penguin));
        round_over = true;
    }

    // TODO: used for debugging, remove
    if keyboard_input.just_pressed(KeyCode::F) {
        let winner_penguin = battle_mode_context
            .leaderboard
            .scores
            .iter()
            .choose(&mut rand::thread_rng())
            .map(|(p, _)| *p)
            .unwrap();
        battle_mode_context.round_outcome = Some(RoundOutcome::Winner(winner_penguin));
        round_over = true;
    }

    if round_over {
        state.overwrite_pop().unwrap();
    }
}

pub fn setup_leaderboard_display(
    mut commands: Commands,
    base_color_materials: Res<BaseColorMaterials>,
    textures: Res<Textures>,
    fonts: Res<Fonts>,
    battle_mode_context: Res<BattleModeContext>,
    windows: Res<Windows>,
    query: Query<Entity, With<UIRoot>>,
) {
    let mut leaderboard_display_box = None;
    let window = windows.get_primary().unwrap();

    commands.entity(query.single()).with_children(|parent| {
        leaderboard_display_box = Some(
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(window.width()), Val::Px(window.height())),
                        position_type: PositionType::Absolute,
                        position: Rect {
                            left: Val::Px(0.0),
                            top: Val::Px(0.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    material: base_color_materials.colors[0].clone(),
                    ..Default::default()
                })
                .insert(UIComponent)
                .with_children(|parent| {
                    // spawn border
                    let mut spawn_color = |y: usize, x: usize| {
                        parent
                            .spawn_bundle(NodeBundle {
                                style: Style {
                                    size: Size::new(
                                        Val::Px(PIXEL_SCALE as f32),
                                        Val::Px(PIXEL_SCALE as f32),
                                    ),
                                    position_type: PositionType::Absolute,
                                    position: Rect {
                                        left: Val::Px((x * PIXEL_SCALE) as f32),
                                        top: Val::Px((y * PIXEL_SCALE) as f32),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                material: base_color_materials.colors[rand::thread_rng()
                                    .gen_range(0..base_color_materials.colors.len())]
                                .clone(),
                                ..Default::default()
                            })
                            .insert(UIComponent);
                    };

                    let height = window.height() as usize / PIXEL_SCALE;
                    let width = window.width() as usize / PIXEL_SCALE;
                    for y in 0..height {
                        spawn_color(y, 0);
                        spawn_color(y, 1);
                        spawn_color(y, width - 2);
                        spawn_color(y, width - 1);
                    }
                    for x in 2..width - 2 {
                        spawn_color(0, x);
                        spawn_color(1, x);
                        spawn_color(height - 2, x);
                        spawn_color(height - 1, x);
                    }

                    for (penguin, score) in &battle_mode_context.leaderboard.scores {
                        // spawn penguin portrait
                        parent
                            .spawn_bundle(NodeBundle {
                                style: Style {
                                    size: Size::new(
                                        Val::Px(TILE_WIDTH as f32),
                                        Val::Px(TILE_HEIGHT as f32),
                                    ),
                                    position_type: PositionType::Absolute,
                                    position: Rect {
                                        left: Val::Px(4.0 * PIXEL_SCALE as f32),
                                        top: Val::Px(((6 + penguin.0 * 12) * PIXEL_SCALE) as f32),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                material: base_color_materials.colors[2].clone(),
                                ..Default::default()
                            })
                            .insert(UIComponent)
                            .with_children(|parent| {
                                parent
                                    .spawn_bundle(ImageBundle {
                                        style: Style {
                                            size: Size::new(
                                                Val::Percent(100.0),
                                                Val::Percent(100.0),
                                            ),
                                            ..Default::default()
                                        },
                                        material: textures.get_penguin_texture(*penguin).clone(),
                                        ..Default::default()
                                    })
                                    .insert(UIComponent);
                            });

                        // spawn penguin trophies
                        for i in 0..*score {
                            parent
                                .spawn_bundle(ImageBundle {
                                    style: Style {
                                        size: Size::new(
                                            Val::Px(5.0 * PIXEL_SCALE as f32),
                                            Val::Px(7.0 * PIXEL_SCALE as f32),
                                        ),
                                        position_type: PositionType::Absolute,
                                        position: Rect {
                                            top: Val::Px(
                                                ((7 + penguin.0 * 12) * PIXEL_SCALE) as f32,
                                            ),
                                            left: Val::Px(((15 + i * 9) * PIXEL_SCALE) as f32),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    material: textures.trophy.clone(),
                                    ..Default::default()
                                })
                                .insert(UIComponent);
                        }

                        if let RoundOutcome::Winner(round_winner_penguin) =
                            battle_mode_context.round_outcome.unwrap()
                        {
                            if *penguin == round_winner_penguin {
                                let mut place_text = |y, x, str: &str, c: usize| {
                                    parent
                                        .spawn_bundle(TextBundle {
                                            text: Text::with_section(
                                                str.to_string(),
                                                TextStyle {
                                                    font: fonts.mono.clone(),
                                                    font_size: 2.0 * PIXEL_SCALE as f32,
                                                    color: COLORS[c].into(),
                                                },
                                                TextAlignment::default(),
                                            ),
                                            style: Style {
                                                position_type: PositionType::Absolute,
                                                position: Rect {
                                                    top: Val::Px(y as f32 * PIXEL_SCALE as f32),
                                                    left: Val::Px(x as f32 * PIXEL_SCALE as f32),
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        })
                                        .insert(UIComponent);
                                };

                                place_text(6 + penguin.0 * 12, 15 + (*score - 1) * 9 - 2, "*", 15);
                                place_text(8 + penguin.0 * 12, 15 + (*score - 1) * 9 + 6, "*", 15);
                                place_text(10 + penguin.0 * 12, 15 + (*score - 1) * 9 - 1, "*", 15);
                            }
                        }
                    }
                })
                .id(),
        );
    });

    commands.insert_resource(LeaderboardDisplayContext {
        leaderboard_display_box: leaderboard_display_box.unwrap(),
        timer: Timer::from_seconds(1.5, false),
    });
}

pub fn leaderboard_display_update(
    mut commands: Commands,
    time: Res<Time>,
    mut leaderboard_display_context: ResMut<LeaderboardDisplayContext>,
    mut state: ResMut<State<AppState>>,
) {
    leaderboard_display_context.timer.tick(time.delta());
    if leaderboard_display_context.timer.finished() {
        commands
            .entity(leaderboard_display_context.leaderboard_display_box)
            .despawn_recursive();
        commands.remove_resource::<LeaderboardDisplayContext>();
        state.pop().unwrap();
    }
}

pub fn teardown(
    mut commands: Commands,
    query: Query<Entity>,
    mut player_action_events: ResMut<Events<PlayerActionEvent>>,
    mut explosion_events: ResMut<Events<ExplosionEvent>>,
    mut burn_events: ResMut<Events<BurnEvent>>,
    mut damage_events: ResMut<Events<DamageEvent>>,
) {
    // clear entities
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // clear events
    player_action_events.clear();
    explosion_events.clear();
    burn_events.clear();
    damage_events.clear();

    // clear resources
    // common
    commands.remove_resource::<GameContext>();
    commands.remove_resource::<GameTimer>();
    commands.remove_resource::<WorldID>();
    commands.remove_resource::<MapSize>();

    // battle mode
    commands.remove_resource::<BattleModeContext>();
    commands.remove_resource::<WallOfDeath>();
}
