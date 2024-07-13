use std::time::Duration;

use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
    window::PrimaryWindow,
};
use rand::{
    prelude::{IteratorRandom, SliceRandom},
    Rng,
};

use crate::{
    audio::Audio,
    common::{
        constants::{COLORS, PIXEL_SCALE},
        resources::{Fonts, InputActionStatusTracker},
        types::InputAction,
    },
    AppState,
};

use super::{
    ai::*,
    components::*,
    constants::*,
    events::*,
    resources::*,
    types::{Direction, *},
    utils::*,
};

pub fn resize_window(
    mut primary_query: Query<&mut Window, With<PrimaryWindow>>,
    map_size: Res<MapSize>,
) {
    primary_query.get_single_mut().unwrap().resolution.set(
        (map_size.columns * TILE_WIDTH) as f32,
        (HUD_HEIGHT + map_size.rows * TILE_HEIGHT) as f32,
    );
}

pub fn spawn_cameras(mut commands: Commands, map_size: Res<MapSize>) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(
            ((map_size.columns * TILE_WIDTH) as f32) / 2.0,
            -((map_size.rows * TILE_HEIGHT - HUD_HEIGHT) as f32 / 2.0),
            999.9,
        ),
        ..default()
    });
}

pub fn setup_penguin_portraits(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    hud_colors: Res<HUDColors>,
    query: Query<Entity, With<PenguinPortraitDisplay>>,
    query2: Query<&Penguin>,
) {
    if let Ok(e) = query.get_single() {
        let penguin_tags = query2.iter().copied().collect::<Vec<Penguin>>();
        commands.entity(e).with_children(|parent| {
            init_penguin_portraits(parent, &penguin_tags, &hud_colors, &game_textures);
        });
    }
}

pub fn hud_update(
    mut commands: Commands,
    game_timer: Res<GameTimer>,
    mut query: Query<&mut Text, With<GameTimerDisplay>>,
    query2: Query<&Penguin>,
    query3: Query<(Entity, &PenguinPortrait)>,
) {
    let remaining_seconds = (game_timer.0.duration() - game_timer.0.elapsed())
        .as_secs_f32()
        .ceil() as usize;
    query.single_mut().sections[0].value = format_hud_time(remaining_seconds);

    // remove dead penguin portraits :(
    for (entity, PenguinPortrait(penguin)) in query3.iter() {
        if !query2.iter().any(|p| p.0 == penguin.0) {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn hud_indicate_pause(mut query: Query<&mut Text, With<GameTimerDisplay>>) {
    query.single_mut().sections[0].value = String::from("PAUSE");
}

pub fn move_cooldown_tick(time: Res<Time>, mut query: Query<&mut MoveCooldown>) {
    for mut move_cooldown in query.iter_mut() {
        move_cooldown.0.tick(time.delta());
    }
}

pub fn game_timer_tick(time: Res<Time>, mut game_timer: ResMut<GameTimer>) {
    game_timer.0.tick(time.delta());
}

pub fn handle_user_input(
    mut commands: Commands,
    audio: Res<Audio>,
    sounds: Res<Sounds>,
    inputs: Res<InputActionStatusTracker>,
    game_context: Res<GameContext>,
    query: Query<(Entity, &HumanControlled), With<Player>>,
    mut ev_player_action: EventWriter<PlayerActionEvent>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (entity, _) in query.iter().filter(|(_, hc)| hc.0 == 0) {
        for (input_action, direction) in [
            (InputAction::Up, Direction::Up),
            (InputAction::Down, Direction::Down),
            (InputAction::Left, Direction::Left),
            (InputAction::Right, Direction::Right),
        ] {
            if inputs.is_active(input_action) {
                ev_player_action.send(PlayerActionEvent {
                    player: entity,
                    action: PlayerAction::Move(direction),
                });
            }
        }

        if inputs.is_active(InputAction::Space) {
            ev_player_action.send(PlayerActionEvent {
                player: entity,
                action: PlayerAction::DropBomb,
            });
        }
    }

    for (entity, _) in query.iter().filter(|(_, hc)| hc.0 == 1) {
        for (input_action, direction) in [
            (InputAction::W, Direction::Up),
            (InputAction::S, Direction::Down),
            (InputAction::A, Direction::Left),
            (InputAction::D, Direction::Right),
        ] {
            if inputs.is_active(input_action) {
                ev_player_action.send(PlayerActionEvent {
                    player: entity,
                    action: PlayerAction::Move(direction),
                });
            }
        }

        if inputs.is_active(InputAction::G) {
            ev_player_action.send(PlayerActionEvent {
                player: entity,
                action: PlayerAction::DropBomb,
            });
        }
    }

    if inputs.is_active(InputAction::Return) && game_context.pausable {
        audio.play(sounds.pause);
        // TODO: this resource can leak if, for example, the game is over in the same frame and the state change gets overriden
        commands.insert_resource(PauseContext {
            in_game_state: *state.get(),
        });
        next_state.set(AppState::Paused);
    }

    if inputs.is_active(InputAction::Escape) {
        next_state.set(game_context.game_mode_manager_state);
    }
}

pub fn mob_ai(
    mut query: Query<(Entity, &Position, &mut MobAI, Option<&WallHack>), With<Player>>,
    query2: Query<(&Position, Option<&Destructible>), With<Solid>>,
    mut ev_player_action: EventWriter<PlayerActionEvent>,
) {
    let solids: HashMap<Position, bool> = query2.iter().map(|(p, d)| (*p, d.is_some())).collect();

    for (entity, position, mut mob_ai, wall_hack) in query.iter_mut() {
        let mut potential_directions: HashSet<Direction> =
            Direction::LIST.iter().copied().collect();

        if let Some(direction) = mob_ai.direction {
            let result = solids.get(&position.offset(direction, 1));
            if result.is_none() || (wall_hack.is_some() && matches!(result, Some(true))) {
                ev_player_action.send(PlayerActionEvent {
                    player: entity,
                    action: PlayerAction::Move(direction),
                });
            } else {
                mob_ai.direction = None;
                potential_directions.remove(&direction);
            }
        }

        if mob_ai.direction.is_none() {
            // pick potential directions in random order
            let mut potential_directions: Vec<Direction> =
                potential_directions.into_iter().collect();
            potential_directions.shuffle(&mut rand::thread_rng());

            // move towards one that leads to passable terrain (if existing)
            let passable_dir = potential_directions.into_iter().find(|direction| {
                let result = solids.get(&position.offset(*direction, 1));
                result.is_none() || (wall_hack.is_some() && matches!(result, Some(true)))
            });
            if let Some(direction) = passable_dir {
                mob_ai.direction = passable_dir;
                ev_player_action.send(PlayerActionEvent {
                    player: entity,
                    action: PlayerAction::Move(direction),
                });
            }
        }
    }
}

// TODO: most of this function and the contents of the `ai.rs` file are a lazy port which could use some cleaning
pub fn bot_ai(
    query: Query<
        (
            Entity,
            &BotAI,
            &Position,
            &MoveCooldown,
            Option<&WallHack>,
            Option<&BombPush>,
            &BombSatchel,
            &TeamID,
        ),
        With<Player>,
    >,
    query2: Query<&Position, With<Fire>>,
    query3: Query<&Position, With<Bomb>>,
    query4: Query<(&Position, Option<&Destructible>), With<Solid>>,
    query5: Query<&Position, Or<(With<Solid>, With<Exit>)>>,
    query6: Query<(Entity, &Position, &TeamID), With<Player>>,
    query7: Query<&Position, Or<(With<Wall>, With<Bomb>, With<Exit>, With<BurningItem>)>>,
    query8: Query<&Position, With<Destructible>>,
    query9: Query<&Position, (With<Wall>, Without<Destructible>)>,
    query10: Query<&Position, Or<(With<Solid>, With<Item>, With<Player>, With<Exit>)>>,
    query11: Query<&Position, With<Item>>,
    map_size: Res<MapSize>,
    wall_of_death: Option<Res<WallOfDeath>>,
    mut ev_player_action: EventWriter<PlayerActionEvent>,
) {
    // TODO: this is wasted work for situations where there aren't any bots
    let mut rng = rand::thread_rng();
    let fire_positions: HashSet<Position> = query2.iter().copied().collect();
    let bomb_positions: HashSet<Position> = query3.iter().copied().collect();
    let fireproof_positions: HashSet<Position> = query5.iter().copied().collect();
    let invalid_bomb_spawn_positions: HashSet<Position> = query7.iter().copied().collect();
    let destructible_positions: HashSet<Position> = query8.iter().copied().collect();
    let stone_wall_positions: HashSet<Position> = query9.iter().copied().collect();
    let moving_object_stoppers: HashSet<Position> = query10.iter().copied().collect();
    let item_positions: HashSet<Position> = query11.iter().copied().collect();

    let wall_of_death = wall_of_death.as_deref();

    for (entity, bot_ai, position, move_cooldown, wall_hack, bomb_push, bomb_satchel, team_id) in
        query.iter()
    {
        let impassable_positions: HashSet<Position> = if wall_hack.is_none() {
            query4.iter().map(|(p, _)| *p).collect()
        } else {
            query4
                .iter()
                .filter_map(|(p, d)| if d.is_some() { None } else { Some(*p) })
                .collect()
        };

        let enemy_positions: Vec<Position> = query6
            .iter()
            .filter(|(_, _, tid)| tid.0 != team_id.0)
            .map(|(_, p, _)| *p)
            .collect();

        let bot_difficulty = bot_ai.difficulty;
        let assumed_bomb_range = bomb_satchel.bomb_range + 2;

        // miss?
        match bot_difficulty {
            BotDifficulty::Easy | BotDifficulty::Medium => {
                if rng.gen_range(0..100)
                    < match bot_difficulty {
                        BotDifficulty::Easy => 30,
                        BotDifficulty::Medium => 15,
                        BotDifficulty::Hard => unreachable!(),
                    }
                {
                    continue;
                }
            }
            BotDifficulty::Hard => (),
        }

        let command_priority_list = [0, 3, 6, 1, 4, 2, 5, 7];
        let mut action = None;
        let mut bomb_flag = 0;
        let mut nav_flag = -1;
        for mut com in command_priority_list {
            if action.is_some() {
                break;
            }

            // miss?
            match bot_difficulty {
                BotDifficulty::Easy | BotDifficulty::Medium => {
                    if rng.gen_range(0..100)
                        < match bot_difficulty {
                            BotDifficulty::Easy => 30,
                            BotDifficulty::Medium => 15,
                            BotDifficulty::Hard => unreachable!(),
                        }
                    {
                        com = rng.gen_range(0..8);
                    }
                }
                BotDifficulty::Hard => (),
            }

            match com {
                0 => {
                    if !safe(
                        *position,
                        &fire_positions,
                        &bomb_positions,
                        assumed_bomb_range,
                        &fireproof_positions,
                        wall_of_death,
                        *map_size,
                    ) {
                        action = safe_dir(
                            *position,
                            &fire_positions,
                            &bomb_positions,
                            assumed_bomb_range,
                            &fireproof_positions,
                            &impassable_positions,
                            wall_of_death,
                            *map_size,
                            bomb_push.is_some(),
                            &moving_object_stoppers,
                        )
                        .iter()
                        .choose(&mut rng)
                        .map(|d| (PlayerAction::Move(*d), PlayerIntention::MoveToSafety));
                    }
                }
                1 => {
                    action = detect_powers(
                        *position,
                        &impassable_positions,
                        &fire_positions,
                        &bomb_positions,
                        assumed_bomb_range,
                        &fireproof_positions,
                        wall_of_death,
                        *map_size,
                        &item_positions,
                    )
                    .iter()
                    .choose(&mut rng)
                    .map(|d| (PlayerAction::Move(*d), PlayerIntention::PickUpItem));
                }
                2 => {
                    action = destroy_blocks(
                        *position,
                        bomb_satchel,
                        &invalid_bomb_spawn_positions,
                        &fire_positions,
                        &bomb_positions,
                        assumed_bomb_range,
                        &fireproof_positions,
                        &impassable_positions,
                        &destructible_positions,
                        wall_of_death,
                        *map_size,
                        bomb_push.is_some(),
                        &moving_object_stoppers,
                    )
                    .map(|a| (a, PlayerIntention::DestroyBlocks));
                }
                3 => {
                    if bomb_flag == 0
                        && bomb_satchel.bombs_available > 0
                        && can_kill(
                            *position,
                            bomb_satchel.bomb_range,
                            &enemy_positions,
                            &stone_wall_positions,
                        )
                        && should_place_bomb(
                            *position,
                            &invalid_bomb_spawn_positions,
                            &bomb_positions,
                            assumed_bomb_range,
                            &fire_positions,
                            &fireproof_positions,
                            &impassable_positions,
                            wall_of_death,
                            *map_size,
                            bomb_push.is_some(),
                            &moving_object_stoppers,
                        )
                    {
                        action = Some((PlayerAction::DropBomb, PlayerIntention::KillPlayers));
                    }
                    bomb_flag = 1;
                }
                4 => {
                    if bomb_flag == 0
                        && bomb_satchel.bombs_available > 0
                        && players_in_range(*position, &enemy_positions, bomb_satchel.bomb_range)
                        && should_place_bomb(
                            *position,
                            &invalid_bomb_spawn_positions,
                            &bomb_positions,
                            assumed_bomb_range,
                            &fire_positions,
                            &fireproof_positions,
                            &impassable_positions,
                            wall_of_death,
                            *map_size,
                            bomb_push.is_some(),
                            &moving_object_stoppers,
                        )
                    {
                        action = Some((
                            PlayerAction::DropBomb,
                            PlayerIntention::PlaceBombNearPlayers,
                        ));
                    }
                    bomb_flag = 1;
                }
                5 => {
                    if nav_flag == -1 {
                        nav_flag = 0;
                    }
                }
                6 => {
                    if nav_flag == -1 && rng.gen_bool(0.125) {
                        let direction = Direction::LIST.choose(&mut rng).unwrap();
                        let position = position.offset(*direction, 1);

                        if !impassable_positions.contains(&position)
                            && safe(
                                position,
                                &fire_positions,
                                &bomb_positions,
                                assumed_bomb_range,
                                &fireproof_positions,
                                wall_of_death,
                                *map_size,
                            )
                        {
                            action =
                                Some((PlayerAction::Move(*direction), PlayerIntention::RandomMove));
                        }
                    }
                }
                7 => {
                    if nav_flag == -1 {
                        nav_flag = 1;
                    }
                }
                _ => unreachable!(),
            }
        }

        if action.is_none() {
            if nav_flag == 0 {
                action = hunt_players(
                    *position,
                    *map_size,
                    &query6
                        .iter()
                        .map(|p| (*p.1, p.0))
                        .collect::<Vec<(Position, Entity)>>(),
                    entity,
                    &stone_wall_positions,
                    &impassable_positions,
                    &fire_positions,
                    &bomb_positions,
                    assumed_bomb_range,
                    &fireproof_positions,
                    wall_of_death,
                )
                .iter()
                .choose(&mut rng)
                .map(|d| (PlayerAction::Move(*d), PlayerIntention::HuntPlayers));
            } else {
                action = flee(
                    *position,
                    &enemy_positions,
                    &impassable_positions,
                    &fire_positions,
                    &bomb_positions,
                    assumed_bomb_range,
                    &fireproof_positions,
                    wall_of_death,
                    *map_size,
                )
                .iter()
                .choose(&mut rng)
                .map(|d| (PlayerAction::Move(*d), PlayerIntention::Flee));
            }
        }

        // don't send meaningless actions
        if matches!(action, Some((PlayerAction::Move(_), _))) && !move_cooldown.0.ready() {
            continue;
        }

        if let Some((action, intention)) = action {
            println!("bot {:?}: {:?} - {:?}", entity, intention, action);

            ev_player_action.send(PlayerActionEvent {
                player: entity,
                action,
            });
        }
    }
}

pub fn player_move(
    mut commands: Commands,
    mut ev_player_action: EventReader<PlayerActionEvent>,
    mut p: ParamSet<(
        Query<
            (
                &mut Position,
                &mut Sprite,
                Option<&WallHack>,
                Option<&BombPush>,
                Option<&mut MoveCooldown>,
            ),
            With<Player>,
        >,
        Query<(
            Entity,
            &Solid,
            &Position,
            Option<&Destructible>,
            Option<&Bomb>,
        )>,
    )>,
    mut query2: Query<&mut Transform>,
) {
    let solids: HashMap<Position, (Entity, bool, bool)> = p
        .p1()
        .iter()
        .map(|(e, _, p, d, b)| (*p, (e, d.is_some(), b.is_some())))
        .collect();

    for (entity, direction) in ev_player_action.read().filter_map(|p| {
        if let PlayerAction::Move(direction) = p.action {
            Some((p.player, direction))
        } else {
            None
        }
    }) {
        if let Ok((mut position, mut sprite, wall_hack, bomb_push, mut move_cooldown)) =
            p.p0().get_mut(entity)
        {
            // visual / sprite flipping
            match direction {
                Direction::Left => sprite.flip_x = true,
                Direction::Right => sprite.flip_x = false,
                _ => (),
            }

            if let Some(move_cooldown) = move_cooldown.as_mut() {
                if !move_cooldown.0.ready() {
                    continue;
                }
            }

            let new_position = position.offset(direction, 1);
            let solid = solids.get(&new_position);

            let mut moved = false;
            if solid.is_none() || (matches!(solid, Some((_, true, _))) && wall_hack.is_some()) {
                *position = new_position;
                moved = true;
            } else if bomb_push.is_some() {
                if let Some((e, _, true)) = solid {
                    commands.entity(*e).insert((
                        Moving { direction },
                        MoveCooldown(Cooldown::from_seconds(0.01)),
                    ));
                }
            }

            if moved {
                if let Some(mut move_cooldown) = move_cooldown {
                    move_cooldown.0.trigger();
                }

                let mut transform = query2.get_mut(entity).unwrap();
                let translation = &mut transform.translation;
                translation.x = get_x(position.x);
                translation.y = get_y(position.y);
            }
        }
    }
}

pub fn moving_object_update(
    mut commands: Commands,
    mut p: ParamSet<(
        Query<(
            Entity,
            &Moving,
            &mut MoveCooldown,
            &mut Position,
            &mut Transform,
        )>,
        Query<&Position, Or<(With<Solid>, With<Item>, With<Player>, With<Exit>)>>,
    )>,
) {
    let moving_object_entities: Vec<Entity> = p.p0().iter_mut().map(|(e, _, _, _, _)| e).collect();

    for entity in moving_object_entities {
        let impassable_positions: HashSet<Position> = p.p1().iter().copied().collect();

        let mut tmp = p.p0();
        let (entity, moving, mut move_cooldown, mut position, mut transform) =
            tmp.get_mut(entity).unwrap();

        if move_cooldown.0.ready() {
            let new_position = position.offset(moving.direction, 1);
            if impassable_positions.get(&new_position).is_none() {
                *position = new_position;

                let translation = &mut transform.translation;
                translation.x = get_x(position.x);
                translation.y = get_y(position.y);

                move_cooldown.0.trigger();
            } else {
                commands.entity(entity).remove::<Moving>();
                commands.entity(entity).remove::<MoveCooldown>();
            }
        }
    }
}

pub fn pick_up_item(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Health, &Position, &mut BombSatchel), With<Player>>,
    query2: Query<(Entity, &Item, &Position)>,
) {
    let mut rng = rand::thread_rng();
    for (ie, i, ip) in query2.iter() {
        if let Some((pe, mut h, _, mut bomb_satchel)) = query
            .iter_mut()
            .filter(|(_, _, pp, _)| **pp == *ip)
            .choose(&mut rng)
        {
            println!("powered up: {:?}", ip);
            match i {
                Item::Upgrade(Upgrade::BombsUp) => bomb_satchel.bombs_available += 1,
                Item::Upgrade(Upgrade::RangeUp) => bomb_satchel.bomb_range += 1,
                Item::Upgrade(Upgrade::LivesUp) => h.lives += 1,
                Item::Power(Power::Immortal) => {
                    commands.entity(pe).insert(Immortal::default());
                }
                Item::Power(Power::WallHack) => {
                    commands.entity(pe).insert(WallHack);
                }
                Item::Power(Power::BombPush) => {
                    commands.entity(pe).insert(BombPush);
                }
            };

            commands.entity(ie).despawn_recursive();
        }
    }
}

pub fn bomb_drop(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    fonts: Res<Fonts>,
    world_id: Res<WorldID>,
    mut ev_player_action: EventReader<PlayerActionEvent>,
    mut query: Query<(&Position, &mut BombSatchel)>,
    query2: Query<&Position, Or<(With<Solid>, With<Exit>, With<BurningItem>)>>,
) {
    for entity in ev_player_action
        .read()
        .filter(|pa| matches!(pa.action, PlayerAction::DropBomb))
        .map(|pa| pa.player)
    {
        if let Ok((position, mut bomb_satchel)) = query.get_mut(entity) {
            if bomb_satchel.bombs_available > 0 && !query2.iter().any(|p| *p == *position) {
                println!("drop bomb: {:?}", position);
                bomb_satchel.bombs_available -= 1;

                commands
                    .spawn((
                        SpriteBundle {
                            texture: game_textures.bomb.clone(),
                            transform: Transform::from_xyz(
                                get_x(position.x),
                                get_y(position.y),
                                25.0,
                            ),
                            sprite: Sprite {
                                custom_size: Some(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        Bomb {
                            owner: Some(entity),
                            range: bomb_satchel.bomb_range,
                            timer: Timer::from_seconds(2.0, TimerMode::Once),
                        },
                        Solid,
                        *position,
                    ))
                    .with_children(|parent| {
                        let fuse_color = COLORS[if world_id.0 == 2 { 12 } else { 14 }].into();

                        let mut text = Text::from_section(
                            '*',
                            TextStyle {
                                font: fonts.mono.clone(),
                                font_size: 2.0 * PIXEL_SCALE as f32,
                                color: fuse_color,
                            },
                        )
                        .with_alignment(TextAlignment::Center);
                        text.sections.push(TextSection {
                            value: "┐\n │".into(),
                            style: TextStyle {
                                font: fonts.mono.clone(),
                                font_size: 2.0 * PIXEL_SCALE as f32,
                                color: COLORS[0].into(),
                            },
                        });

                        parent.spawn((
                            Text2dBundle {
                                text,
                                transform: Transform::from_xyz(
                                    0.0,
                                    TILE_HEIGHT as f32 / 8.0 * 2.0,
                                    0.0,
                                ),
                                ..Default::default()
                            },
                            Fuse {
                                color: fuse_color,
                                animation_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                            },
                        ));
                    });
            }
        }
    }
}

pub fn animate_fuse(
    time: Res<Time>,
    fonts: Res<Fonts>,
    query: Query<&Bomb>,
    mut query2: Query<(&Parent, &mut Text, &mut Fuse, &mut Transform)>,
) {
    for (parent, mut text, mut fuse, mut transform) in query2.iter_mut() {
        fuse.animation_timer.tick(time.delta());
        let percent_left = fuse.animation_timer.percent_left();
        let fuse_char = match percent_left {
            _ if (0.0..0.33).contains(&percent_left) => 'x',
            _ if (0.33..0.66).contains(&percent_left) => '+',
            _ if (0.66..=1.0).contains(&percent_left) => '*',
            _ => unreachable!(),
        };

        let bomb = query.get(parent.get()).unwrap();
        let percent_left = bomb.timer.percent_left();

        match percent_left {
            _ if (0.66..1.0).contains(&percent_left) => {
                text.sections = vec![
                    TextSection {
                        value: fuse_char.into(),
                        style: TextStyle {
                            font: fonts.mono.clone(),
                            font_size: 2.0 * PIXEL_SCALE as f32,
                            color: fuse.color,
                        },
                    },
                    TextSection {
                        value: "┐\n │".into(),
                        style: TextStyle {
                            font: fonts.mono.clone(),
                            font_size: 2.0 * PIXEL_SCALE as f32,
                            color: COLORS[0].into(),
                        },
                    },
                ];
                let translation = &mut transform.translation;
                translation.x = 0.0;
                translation.y = TILE_HEIGHT as f32 / 8.0 * 2.0;
            }
            _ if (0.33..0.66).contains(&percent_left) => {
                text.sections = vec![
                    TextSection {
                        value: fuse_char.into(),
                        style: TextStyle {
                            font: fonts.mono.clone(),
                            font_size: 2.0 * PIXEL_SCALE as f32,
                            color: fuse.color,
                        },
                    },
                    TextSection {
                        value: "\n│".into(),
                        style: TextStyle {
                            font: fonts.mono.clone(),
                            font_size: 2.0 * PIXEL_SCALE as f32,
                            color: COLORS[0].into(),
                        },
                    },
                ];
                let translation = &mut transform.translation;
                translation.x = TILE_WIDTH as f32 / 12.0;
                translation.y = TILE_HEIGHT as f32 / 8.0 * 2.0;
            }
            _ if (0.0..0.33).contains(&percent_left) => {
                text.sections = vec![TextSection {
                    value: fuse_char.into(),
                    style: TextStyle {
                        font: fonts.mono.clone(),
                        font_size: 2.0 * PIXEL_SCALE as f32,
                        color: fuse.color,
                    },
                }];
                let translation = &mut transform.translation;
                translation.x = TILE_WIDTH as f32 / 12.0;
                translation.y = TILE_HEIGHT as f32 / 8.0 * 1.0;
            }
            _ => (),
        }
    }
}

pub fn bomb_tick(time: Res<Time>, mut query: Query<&mut Bomb>) {
    for mut bomb in query.iter_mut() {
        bomb.timer.tick(time.delta());
    }
}

pub fn fire_tick(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Fire)>) {
    for (entity, mut fire) in query.iter_mut() {
        fire.timer.tick(time.delta());
        if fire.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn crumbling_tick(
    mut commands: Commands,
    time: Res<Time>,
    game_textures: Res<GameTextures>,
    game_context: Res<GameContext>,
    exit_position: Option<Res<ExitPosition>>,
    mut query: Query<(Entity, &mut Crumbling, &Position)>,
) {
    for (entity, mut crumbling, position) in query.iter_mut() {
        crumbling.timer.tick(time.delta());

        if crumbling.timer.finished() {
            commands.entity(entity).despawn_recursive();
            if matches!(exit_position, Some(ref p) if p.0 == *position) {
                commands.spawn((
                    SpriteBundle {
                        texture: game_textures.exit.clone(),
                        transform: Transform::from_xyz(get_x(position.x), get_y(position.y), 10.0),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    *position,
                    Exit::default(),
                ));
            } else if rand::thread_rng().gen_range(0.0..1.0) < ITEM_SPAWN_CHANCE {
                generate_item_at_position(
                    *position,
                    &mut commands,
                    &game_textures,
                    game_context.reduced_loot,
                );
            }
        }
    }
}

pub fn burning_item_tick(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut BurningItem)>,
) {
    for (entity, mut burning_item) in query.iter_mut() {
        burning_item.timer.tick(time.delta());
        if burning_item.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn explode_bombs(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    audio: Res<Audio>,
    sounds: Res<Sounds>,
    query: Query<(Entity, &Bomb, &Position)>,
    query1: Query<(Entity, &Position, Option<&Bomb>), Or<(With<Solid>, With<Exit>)>>,
    mut ev_explosion: EventReader<ExplosionEvent>,
    mut ev_bomb_restock: EventWriter<BombRestockEvent>,
    mut ev_burn: EventWriter<BurnEvent>,
) {
    let exploded_bombs: HashSet<Entity> = ev_explosion.read().map(|e| e.bomb).collect();
    let fireproof_positions: HashSet<Position> = query1
        .iter()
        .filter_map(|(e, p, b)| {
            // ignore bombs that went off
            if !matches!(b, Some(b) if b.timer.finished() || exploded_bombs.contains(&e)) {
                Some(p)
            } else {
                None
            }
        })
        .copied()
        .collect();

    let mut sound_played = false;

    for (entity, bomb, position) in query
        .iter()
        .filter(|(e, b, _)| b.timer.finished() || exploded_bombs.contains(e))
    {
        commands.entity(entity).despawn_recursive();

        if let Some(owner) = bomb.owner {
            ev_bomb_restock.send(BombRestockEvent {
                satchel_owner: owner,
            })
        }

        if !sound_played {
            audio.play(sounds.boom);
            sound_played = true;
        }

        let spawn_fire = |commands: &mut Commands, position: Position| {
            commands.spawn((
                SpriteBundle {
                    texture: game_textures.fire.clone(),
                    transform: Transform::from_xyz(get_x(position.x), get_y(position.y), 5.0),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Fire {
                    timer: Timer::from_seconds(0.5, TimerMode::Once),
                },
                position,
            ));
        };

        spawn_fire(&mut commands, *position);
        for direction in Direction::LIST {
            for i in 1..=bomb.range {
                let position = position.offset(direction, i);

                if fireproof_positions.contains(&position) {
                    ev_burn.send(BurnEvent { position });
                    break;
                }

                spawn_fire(&mut commands, position);
            }
        }
    }
}

pub fn bomb_restock(
    mut ev_bomb_restock: EventReader<BombRestockEvent>,
    mut query: Query<&mut BombSatchel>,
) {
    for event in ev_bomb_restock.read() {
        if let Ok(mut bomb_satchel) = query.get_mut(event.satchel_owner) {
            bomb_satchel.bombs_available += 1;
        }
    }
}

pub fn immortality_tick(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Immortal)>,
) {
    for (entity, mut immortal) in query.iter_mut() {
        immortal.timer.tick(time.delta());

        if immortal.timer.just_finished() {
            commands.entity(entity).remove::<Immortal>();
        }
    }
}

pub fn animate_immortality(
    time: Res<Time>,
    mut p: ParamSet<(
        Query<(
            &mut Immortal,
            &mut Handle<Image>,
            &BaseTexture,
            &ImmortalTexture,
        )>,
        Query<(&mut Handle<Image>, &BaseTexture)>,
    )>,
    mut removals: RemovedComponents<Immortal>,
) {
    // animate currently immortal players
    for (mut immortal, mut texture, base_texture, immortal_texture) in p.p0().iter_mut() {
        if !immortal.timer.finished() {
            immortal.animation_timer.tick(time.delta());
            let percent_left = immortal.animation_timer.percent_left();
            match percent_left {
                _ if (0.5..=1.0).contains(&percent_left) => {
                    *texture = immortal_texture.0.clone();
                }
                _ => *texture = base_texture.0.clone(),
            };
        } else {
            *texture = base_texture.0.clone();
        }
    }

    // revert the texture of players that stopped being immortal
    for entity in removals.read() {
        if let Ok((mut texture, base_texture)) = p.p1().get_mut(entity) {
            *texture = base_texture.0.clone();
        }
    }
}

pub fn fire_effect(mut query: Query<&Position, With<Fire>>, mut ev_burn: EventWriter<BurnEvent>) {
    for position in query.iter_mut() {
        ev_burn.send(BurnEvent {
            position: *position,
        });
    }
}

pub fn melee_attack(
    query: Query<(&Position, &TeamID), With<MeleeAttacker>>,
    query2: Query<(Entity, &Position, &TeamID), With<Player>>,
    mut ev_damage: EventWriter<DamageEvent>,
) {
    for (attacker_position, attacker_team_id) in query.iter() {
        for (e, _, _) in query2
            .iter()
            .filter(|(_, p, tid)| **p == *attacker_position && tid.0 != attacker_team_id.0)
        {
            ev_damage.send(DamageEvent { target: e });
        }
    }
}

pub fn player_burn(
    query: Query<(Entity, &Position), (With<Player>, Without<Immortal>)>,
    query2: Query<&Position, With<Wall>>,
    mut ev_burn: EventReader<BurnEvent>,
    mut ev_damage: EventWriter<DamageEvent>,
) {
    for BurnEvent { position } in ev_burn.read() {
        for (pe, player_pos) in query.iter().filter(|(_, pp)| **pp == *position) {
            if query2.iter().any(|wall_pos| *wall_pos == *player_pos) {
                // Anakin, I have the high ground
                continue;
            }

            ev_damage.send(DamageEvent { target: pe });
        }
    }
}

pub fn player_damage(
    mut commands: Commands,
    mut game_score: Option<ResMut<GameScore>>,
    mut query: Query<
        (
            Entity,
            &mut Health,
            &mut Handle<Image>,
            &ImmortalTexture,
            &SpawnPosition,
            &mut Position,
            &mut Transform,
            &mut Sprite,
            Option<&PointValue>,
        ),
        (With<Player>, Without<Immortal>),
    >,
    mut ev_damage: EventReader<DamageEvent>,
    mut ev_player_death_event: EventWriter<PlayerDeathEvent>,
) {
    let mut damaged_players = HashSet::default();

    for DamageEvent { target } in ev_damage.read() {
        if let Ok((
            pe,
            mut health,
            mut texture,
            immortal_texture,
            spawn_position,
            mut position,
            mut transform,
            mut sprite,
            point_value,
        )) = query.get_mut(*target)
        {
            if damaged_players.contains(&pe) {
                continue;
            }
            damaged_players.insert(pe);

            println!("player damaged: {:?}", pe);
            health.health -= 1;

            let mut gain_immortality = false;
            if health.health == 0 {
                println!("player lost a life: {:?}", pe);
                health.lives -= 1;
                if health.lives == 0 {
                    println!("player died from damage: {:?}", pe);
                    commands.entity(pe).despawn_recursive();

                    ev_player_death_event.send(PlayerDeathEvent);

                    // TODO: move to story_mode
                    if let Some(ref mut game_score) = game_score {
                        if let Some(point_value) = point_value {
                            game_score.0 += point_value.0;
                        }
                    }
                } else {
                    health.health = health.max_health;
                    gain_immortality = true;

                    // return to spawn
                    *position = spawn_position.0;
                    let translation = &mut transform.translation;
                    translation.x = get_x(position.x);
                    translation.y = get_y(position.y);
                    sprite.flip_x = false;

                    // lose powers
                    commands.entity(pe).remove::<BombPush>();
                    commands.entity(pe).remove::<WallHack>();
                }
            } else {
                gain_immortality = true;
            }

            if gain_immortality {
                commands.entity(pe).insert(Immortal::default());
                *texture = immortal_texture.0.clone();
            }
        }
    }
}

pub fn bomb_burn(mut query: Query<(&mut Bomb, &Position)>, mut ev_burn: EventReader<BurnEvent>) {
    for BurnEvent { position } in ev_burn.read() {
        query
            .iter_mut()
            .filter(|(_, p)| **p == *position)
            .for_each(|(mut b, _)| {
                const SHORTENED_FUSE_DURATION: Duration = Duration::from_millis(50);
                if b.timer.duration() - b.timer.elapsed() > SHORTENED_FUSE_DURATION {
                    b.timer.set_duration(SHORTENED_FUSE_DURATION);
                    b.timer.reset();
                }
            });
    }
}

pub fn destructible_wall_burn(
    game_textures: Res<GameTextures>,
    mut commands: Commands,
    mut query: Query<
        (Entity, &Position, &mut Handle<Image>, Option<&Crumbling>),
        (With<Wall>, With<Destructible>),
    >,
    mut ev_burn: EventReader<BurnEvent>,
) {
    for BurnEvent { position } in ev_burn.read() {
        for (e, _, mut t, perishable) in query.iter_mut().filter(|(_, p, _, _)| **p == *position) {
            if perishable.is_none() {
                commands.entity(e).insert(Crumbling {
                    timer: Timer::from_seconds(0.5, TimerMode::Once),
                });
                *t = game_textures.get_map_textures().burning_wall.clone();
            }
        }
    }
}

pub fn item_burn(
    game_textures: Res<GameTextures>,
    mut commands: Commands,
    mut query: Query<(Entity, &Position), With<Item>>,
    mut ev_burn: EventReader<BurnEvent>,
) {
    let mut burned = HashSet::default();

    for BurnEvent { position } in ev_burn.read() {
        for e in query
            .iter_mut()
            .filter(|(_, p)| **p == *position)
            .map(|(e, _)| e)
        {
            if burned.contains(&e) {
                continue;
            }
            burned.insert(e);

            println!("burned item: {:?}", position);

            commands.entity(e).despawn_recursive();
            // burning item
            commands.spawn((
                SpriteBundle {
                    texture: game_textures.burning_item.clone(),
                    transform: Transform::from_xyz(get_x(position.x), get_y(position.y), 20.0),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                *position,
                BurningItem {
                    timer: Timer::from_seconds(0.5, TimerMode::Once),
                },
            ));
        }
    }
}

pub fn exit_burn(
    time: Res<Time>,
    game_textures: Res<GameTextures>,
    mut commands: Commands,
    mut query: Query<(&Position, &mut Exit)>,
    mut ev_burn: EventReader<BurnEvent>,
) {
    // we do checks here because some levels don't have exits (e.g. boss rooms)
    if let Ok((_, mut exit)) = query.get_single_mut() {
        exit.spawn_cooldown.tick(time.delta());
    }

    for BurnEvent { position } in ev_burn.read() {
        if let Ok((exit_position, mut exit)) = query.get_single_mut() {
            if *exit_position == *position && exit.spawn_cooldown.ready() {
                println!("exit burned: {:?}", position);

                // spawn mob
                let base_texture = game_textures.crook.clone();
                commands.spawn((
                    SpriteBundle {
                        texture: base_texture.clone(),
                        transform: Transform::from_xyz(
                            get_x(exit_position.x),
                            get_y(exit_position.y),
                            50.0,
                        ),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    BaseTexture(base_texture),
                    ImmortalTexture(game_textures.immortal_crook.clone()),
                    Player,
                    MobAI::default(),
                    MoveCooldown(Cooldown::from_seconds(0.4)),
                    Health {
                        lives: 1,
                        max_health: 1,
                        health: 1,
                    },
                    *exit_position,
                    SpawnPosition(*exit_position),
                    MeleeAttacker,
                    TeamID(1),
                    Immortal::default(),
                ));

                exit.spawn_cooldown.trigger();
            }
        }
    }
}

pub fn wall_of_death_update(
    mut commands: Commands,
    time: Res<Time>,
    game_textures: Res<GameTextures>,
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

        commands.spawn((
            SpriteBundle {
                texture: game_textures.get_map_textures().wall.clone(),
                transform: Transform::from_xyz(get_x(position.x), get_y(position.y), 10.0),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                    ..Default::default()
                },
                ..Default::default()
            },
            Wall,
            Solid,
            position,
        ));
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

pub fn unpause_on_enter(
    inputs: Res<InputActionStatusTracker>,
    pause_context: Res<PauseContext>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if inputs.is_active(InputAction::Return) {
        next_state.set(pause_context.in_game_state);
    }
}

pub fn quit_game_on_esc(
    inputs: Res<InputActionStatusTracker>,
    game_context: Res<GameContext>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if inputs.is_active(InputAction::Escape) {
        next_state.set(game_context.game_mode_manager_state);
    }
}

pub fn pause_teardown(mut commands: Commands) {
    commands.remove_resource::<PauseContext>();
}
