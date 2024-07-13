use bevy::{
    ecs::event::Events, prelude::*, render::camera::Camera, utils::HashSet, window::PrimaryWindow,
};
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
        utils::*,
    },
    map_transition::MapTransitionInput,
    AppState,
};

use super::{
    constants::BATTLE_MODE_ROUND_DURATION_SECS, resources::*, types::PenguinControlType, utils::*,
};

pub fn setup_battle_mode(
    mut commands: Commands,
    mut game_textures: ResMut<GameTextures>,
    fonts: Res<Fonts>,
    hud_colors: Res<HUDColors>,
    battle_mode_configuration: Res<BattleModeConfiguration>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let world_id = WorldID(rand::thread_rng().gen_range(1..=3));
    game_textures.set_map_textures(world_id);

    let (map_size, percent_of_passable_positions_to_fill) = get_battle_mode_map_size_fill(
        battle_mode_configuration.amount_of_players + battle_mode_configuration.amount_of_bots,
    );

    // spawn HUD
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..Default::default()
                },
                background_color: Color::NONE.into(),
                ..Default::default()
            },
            UIRoot,
            UIComponent,
        ))
        .with_children(|parent| {
            init_hud(
                parent,
                &hud_colors,
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
        bot_difficulty: battle_mode_configuration.bot_difficulty,
        leaderboard,
        round_outcome: None,
        percent_of_passable_positions_to_fill,
    });

    let game_mode_manager_state = AppState::BattleModeManager;
    commands.insert_resource(GameContext {
        pausable: battle_mode_configuration.amount_of_players > 0,
        reduced_loot: true,
        game_mode_manager_state,
    });
    commands.insert_resource(world_id);
    commands.insert_resource(map_size);

    commands.remove_resource::<BattleModeConfiguration>();

    next_state.set(game_mode_manager_state);
}

pub fn battle_mode_manager(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    map_size: Res<MapSize>,
    mut battle_mode_context: ResMut<BattleModeContext>,
    game_option_store: Res<GameOptionStore>,
    mut next_state: ResMut<NextState<AppState>>,
    query: Query<Entity, (Without<Window>, Without<Camera>, Without<UIComponent>)>,
    query2: Query<Entity, With<PenguinPortrait>>,
    mut query4: Query<&mut Text, With<GameTimerDisplay>>,
) {
    loop {
        match battle_mode_context.state {
            BattleModeState::RoundSetup => {
                // map generation //
                let player_spawn_positions = spawn_battle_mode_players(
                    &mut commands,
                    &game_textures,
                    *map_size,
                    &battle_mode_context.players,
                    battle_mode_context.bot_difficulty,
                );

                let wall_entity_reveal_groups = spawn_map(
                    &mut commands,
                    &game_textures,
                    *map_size,
                    battle_mode_context.percent_of_passable_positions_to_fill,
                    true,
                    &player_spawn_positions,
                    &[],
                    false,
                );

                commands.insert_resource(GameTimer(Timer::from_seconds(
                    BATTLE_MODE_ROUND_DURATION_SECS as f32,
                    TimerMode::Once,
                )));
                // update HUD clock
                query4.single_mut().sections[0].value =
                    format_hud_time(BATTLE_MODE_ROUND_DURATION_SECS);
                commands.insert_resource(WallOfDeath::Dormant(Timer::from_seconds(
                    BATTLE_MODE_ROUND_DURATION_SECS as f32 / 2.0,
                    TimerMode::Once,
                )));

                battle_mode_context.round_outcome = None;

                if game_option_store.get(GameOption::Transition) {
                    battle_mode_context.state = BattleModeState::MapTransition;
                    commands.insert_resource(MapTransitionInput {
                        wall_entity_reveal_groups,
                        next_state: AppState::BattleModeManager,
                    });
                    next_state.set(AppState::MapTransition);
                } else {
                    start_round(battle_mode_context, commands, next_state);
                }
            }
            BattleModeState::MapTransition => {
                start_round(battle_mode_context, commands, next_state)
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
                            commands.entity(entity).despawn();
                        }

                        // clear penguin portraits
                        for entity in query2.iter() {
                            commands.entity(entity).despawn_recursive();
                        }

                        battle_mode_context.state = BattleModeState::LeaderboardDisplay;
                        next_state.set(AppState::LeaderboardDisplay);
                    }
                    None => {
                        // abrupt exit
                        next_state.set(AppState::BattleModeTeardown);
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
                    next_state.set(AppState::BattleModeTeardown);
                } else {
                    battle_mode_context.state = BattleModeState::RoundSetup;
                    continue;
                }
            }
        }
        break;
    }
}

pub fn finish_freeze(
    mut commands: Commands,
    time: Res<Time>,
    mut freeze_timer: ResMut<FreezeTimer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    freeze_timer.0.tick(time.delta());
    if freeze_timer.0.finished() {
        commands.remove_resource::<FreezeTimer>();
        next_state.set(AppState::BattleModeInGame);
    }
}

pub fn on_death_item_pinata(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
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
    for _ in ev_player_death_event.read() {
        let invalid_positions: HashSet<Position> = query.iter().copied().collect();
        let valid_positions = (0..map_size.rows)
            .flat_map(|y| {
                (0..map_size.columns).map(move |x| Position {
                    y: y as isize,
                    x: x as isize,
                })
            })
            .filter(|p| !invalid_positions.contains(p));
        for position in valid_positions.choose_multiple(&mut rand::thread_rng(), 3) {
            generate_item_at_position(
                position,
                &mut commands,
                &game_textures,
                game_context.reduced_loot,
            );
        }
    }
}

pub fn finish_round(
    game_timer: Res<GameTimer>,
    mut battle_mode_context: ResMut<BattleModeContext>,
    query: Query<&Penguin, With<Player>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let mut round_over = false;
    if game_timer.0.finished() || query.iter().count() == 0 {
        battle_mode_context.round_outcome = Some(RoundOutcome::Tie);
        round_over = true;
    } else if let Ok(penguin) = query.get_single() {
        battle_mode_context.round_outcome = Some(RoundOutcome::Winner(*penguin));
        round_over = true;
    }

    if round_over {
        next_state.set(AppState::BattleModeManager);
    }
}

pub fn setup_leaderboard_display(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    leaderboard_textures: Res<LeaderboardTextures>,
    fonts: Res<Fonts>,
    battle_mode_context: Res<BattleModeContext>,
    primary_query: Query<&Window, With<PrimaryWindow>>,
    query: Query<Entity, With<UIRoot>>,
) {
    let mut leaderboard_display_box = None;
    let window = primary_query.get_single().unwrap();

    commands.entity(query.single()).with_children(|parent| {
        leaderboard_display_box = Some(
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            top: Val::Px(0.0),
                            width: Val::Px(window.width()),
                            height: Val::Px(window.height()),
                            ..Default::default()
                        },
                        background_color: COLORS[0].into(),
                        ..Default::default()
                    },
                    UIComponent,
                ))
                .with_children(|parent| {
                    // spawn border
                    let mut spawn_color = |y: usize, x: usize| {
                        parent.spawn((
                            NodeBundle {
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px((x * PIXEL_SCALE) as f32),
                                    top: Val::Px((y * PIXEL_SCALE) as f32),
                                    width: Val::Px(PIXEL_SCALE as f32),
                                    height: Val::Px(PIXEL_SCALE as f32),
                                    ..Default::default()
                                },
                                background_color: (*COLORS
                                    .iter()
                                    .choose(&mut rand::thread_rng())
                                    .unwrap())
                                .into(),
                                ..Default::default()
                            },
                            UIComponent,
                        ));
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
                            .spawn((
                                NodeBundle {
                                    style: Style {
                                        position_type: PositionType::Absolute,
                                        left: Val::Px(4.0 * PIXEL_SCALE as f32),
                                        top: Val::Px(((6 + penguin.0 * 12) * PIXEL_SCALE) as f32),
                                        width: Val::Px(TILE_WIDTH as f32),
                                        height: Val::Px(TILE_HEIGHT as f32),
                                        ..Default::default()
                                    },
                                    background_color: COLORS[2].into(),
                                    ..Default::default()
                                },
                                UIComponent,
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    ImageBundle {
                                        style: Style {
                                            width: Val::Percent(100.0),
                                            height: Val::Percent(100.0),
                                            ..Default::default()
                                        },
                                        image: game_textures
                                            .get_penguin_texture(*penguin)
                                            .clone()
                                            .into(),
                                        ..Default::default()
                                    },
                                    UIComponent,
                                ));
                            });

                        // spawn penguin trophies
                        for i in 0..*score {
                            parent.spawn((
                                ImageBundle {
                                    style: Style {
                                        position_type: PositionType::Absolute,
                                        top: Val::Px(((7 + penguin.0 * 12) * PIXEL_SCALE) as f32),
                                        left: Val::Px(((15 + i * 9) * PIXEL_SCALE) as f32),
                                        width: Val::Px(5.0 * PIXEL_SCALE as f32),
                                        height: Val::Px(7.0 * PIXEL_SCALE as f32),
                                        ..Default::default()
                                    },
                                    image: leaderboard_textures.trophy.clone().into(),
                                    ..Default::default()
                                },
                                UIComponent,
                            ));
                        }

                        if let RoundOutcome::Winner(round_winner_penguin) =
                            battle_mode_context.round_outcome.unwrap()
                        {
                            if *penguin == round_winner_penguin {
                                let mut place_text = |y, x, str: &str, c: usize| {
                                    parent.spawn((
                                        TextBundle {
                                            text: Text::from_section(
                                                str.to_string(),
                                                TextStyle {
                                                    font: fonts.mono.clone(),
                                                    font_size: 2.0 * PIXEL_SCALE as f32,
                                                    color: COLORS[c].into(),
                                                },
                                            ),
                                            style: Style {
                                                position_type: PositionType::Absolute,
                                                top: Val::Px(y as f32 * PIXEL_SCALE as f32),
                                                left: Val::Px(x as f32 * PIXEL_SCALE as f32),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        UIComponent,
                                    ));
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
        timer: Timer::from_seconds(1.5, TimerMode::Once),
    });
}

pub fn leaderboard_display_update(
    mut commands: Commands,
    time: Res<Time>,
    mut leaderboard_display_context: ResMut<LeaderboardDisplayContext>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    leaderboard_display_context.timer.tick(time.delta());
    if leaderboard_display_context.timer.finished() {
        commands
            .entity(leaderboard_display_context.leaderboard_display_box)
            .despawn_recursive();
        commands.remove_resource::<LeaderboardDisplayContext>();
        next_state.set(AppState::BattleModeManager);
    }
}

pub fn teardown(
    mut commands: Commands,
    query: Query<Entity, Without<Window>>,
    mut player_action_events: ResMut<Events<PlayerActionEvent>>,
    mut explosion_events: ResMut<Events<ExplosionEvent>>,
    mut burn_events: ResMut<Events<BurnEvent>>,
    mut damage_events: ResMut<Events<DamageEvent>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // clear entities
    for entity in query.iter() {
        commands.entity(entity).despawn();
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

    next_state.set(AppState::MainMenu);
}
