use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use bevy::{
    prelude::*,
    render::camera::{Camera, VisibleEntities},
};
use bevy_kira_audio::Audio;
use rand::{
    prelude::{IteratorRandom, SliceRandom},
    Rng,
};

use crate::{
    common::{
        constants::{COLORS, PIXEL_SCALE},
        resources::Fonts,
    },
    AppState,
};

use super::{
    camera::SimpleOrthoProjection,
    components::*,
    constants::*,
    events::*,
    resources::*,
    types::{Direction, *},
    utils::*,
};

pub fn resize_window(mut windows: ResMut<Windows>, map_size: Res<MapSize>) {
    let window = windows.get_primary_mut().unwrap();
    window.set_resolution(
        (map_size.columns * TILE_WIDTH) as f32,
        (HUD_HEIGHT + map_size.rows * TILE_HEIGHT) as f32,
    );
}

pub fn spawn_cameras(mut commands: Commands, map_size: Res<MapSize>) {
    let projection = SimpleOrthoProjection::new(
        (map_size.rows * TILE_HEIGHT) as f32,
        (map_size.columns * TILE_WIDTH) as f32,
    );
    let cam_name = bevy::render::render_graph::base::camera::CAMERA_2D;
    let camera = Camera {
        name: Some(cam_name.to_string()),
        ..Default::default()
    };

    commands.spawn_bundle((
        Transform::from_translation(Vec3::new(0.0, 0.0, projection.far - 0.1)),
        GlobalTransform::default(),
        VisibleEntities::default(),
        camera,
        projection,
    ));

    commands.spawn_bundle(UiCameraBundle::default());
}

pub fn setup_penguin_portraits(
    mut commands: Commands,
    textures: Res<Textures>,
    hud_materials: Res<HUDMaterials>,
    query: Query<Entity, With<PenguinPortraitDisplay>>,
    query2: Query<&Penguin>,
) {
    if let Ok(e) = query.single() {
        let penguin_tags = query2.iter().copied().collect::<Vec<Penguin>>();
        commands.entity(e).with_children(|parent| {
            init_penguin_portraits(parent, &penguin_tags, &hud_materials, &textures);
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
    query.single_mut().unwrap().sections[0].value = format_hud_time(remaining_seconds);

    // remove dead penguin portraits :(
    for (entity, PenguinPortrait(penguin)) in query3.iter() {
        if !query2.iter().any(|p| p.0 == penguin.0) {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn hud_indicate_pause(mut query: Query<&mut Text, With<GameTimerDisplay>>) {
    query.single_mut().unwrap().sections[0].value = String::from("PAUSE");
}

pub fn move_cooldown_tick(time: Res<Time>, mut query: Query<&mut MoveCooldown>) {
    for mut move_cooldown in query.iter_mut() {
        move_cooldown.0.tick(time.delta());
    }
}

pub fn game_timer_tick(time: Res<Time>, mut game_timer: ResMut<GameTimer>) {
    game_timer.0.tick(time.delta());
}

pub fn handle_keyboard_input(
    audio: Res<Audio>,
    sounds: Res<Sounds>,
    mut keyboard_input: ResMut<Input<KeyCode>>,
    game_context: Res<GameContext>,
    query: Query<(Entity, &HumanControlled), With<Player>>,
    mut ev_player_action: EventWriter<PlayerActionEvent>,
    mut state: ResMut<State<AppState>>,
) {
    for (entity, _) in query.iter().filter(|(_, hc)| hc.0 == 0) {
        for (key_code, direction) in [
            (KeyCode::Up, Direction::Up),
            (KeyCode::Down, Direction::Down),
            (KeyCode::Left, Direction::Left),
            (KeyCode::Right, Direction::Right),
        ] {
            if keyboard_input.just_pressed(key_code) {
                ev_player_action.send(PlayerActionEvent(entity, PlayerAction::Move(direction)));
            }
        }

        if keyboard_input.just_pressed(KeyCode::Space) {
            ev_player_action.send(PlayerActionEvent(entity, PlayerAction::DropBomb));
        }
    }

    if keyboard_input.just_pressed(KeyCode::Return) && game_context.pausable {
        audio.stop();
        audio.play(sounds.pause.clone());
        state.push(AppState::Paused).unwrap();
        keyboard_input.reset(KeyCode::Return);
    }

    if keyboard_input.just_pressed(KeyCode::Escape) {
        state.overwrite_pop().unwrap();
        keyboard_input.reset(KeyCode::Escape);
    }
}

// demo for touchscreen testing
pub fn handle_mouse_input(
    mouse_button_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    query: Query<(Entity, &HumanControlled), With<Player>>,
    mut ev_player_action: EventWriter<PlayerActionEvent>,
) {
    for (entity, _) in query.iter().filter(|(_, hc)| hc.0 == 0) {
        if mouse_button_input.just_pressed(MouseButton::Left) {
            let window = windows.get_primary().unwrap();

            if let Some(position) = window.cursor_position() {
                let width = window.width();
                let height = window.height();

                let scale_x = position.x / width;
                let scale_y = position.y / height;

                println!(
                    "mouse click: {:?} / w: {}, h: {} / scale_x: {}, scale_y: {}",
                    position, width, height, scale_x, scale_y
                );

                if scale_x < 0.25 {
                    ev_player_action.send(PlayerActionEvent(
                        entity,
                        PlayerAction::Move(Direction::Left),
                    ))
                }
                if scale_x >= 0.75 {
                    ev_player_action.send(PlayerActionEvent(
                        entity,
                        PlayerAction::Move(Direction::Right),
                    ))
                }

                if scale_y < 0.25 {
                    ev_player_action.send(PlayerActionEvent(
                        entity,
                        PlayerAction::Move(Direction::Down),
                    ))
                }
                if scale_y >= 0.75 {
                    ev_player_action
                        .send(PlayerActionEvent(entity, PlayerAction::Move(Direction::Up)))
                }

                if (0.25..0.75).contains(&scale_x) && (0.25..0.75).contains(&scale_y) {
                    ev_player_action.send(PlayerActionEvent(entity, PlayerAction::DropBomb));
                }
            }
        }
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
                ev_player_action.send(PlayerActionEvent(entity, PlayerAction::Move(direction)));
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
                ev_player_action.send(PlayerActionEvent(entity, PlayerAction::Move(direction)));
            }
        }
    }
}

pub fn bot_ai(
    query: Query<
        (
            Entity,
            &Position,
            &MoveCooldown,
            Option<&WallHack>,
            &BombSatchel,
            &TeamID,
        ),
        (With<Player>, With<BotAI>),
    >,
    query2: Query<&Position, With<Fire>>,
    query3: Query<(&Position, &Bomb)>,
    query4: Query<(&Position, Option<&Destructible>), With<Solid>>,
    query5: Query<&Position, Or<(With<Solid>, With<Exit>)>>,
    query6: Query<(&Position, &TeamID), With<Player>>,
    query7: Query<&Position, With<Wall>>,
    query8: Query<&Position, Or<(With<Wall>, With<Bomb>, With<Exit>, With<BurningItem>)>>,
    query9: Query<&Position, With<Destructible>>,
    mut ev_player_action: EventWriter<PlayerActionEvent>,
) {
    // TODO: this is wasted work for situations where there aren't any bots
    let mut rng = rand::thread_rng();
    let fire_positions: HashSet<Position> = query2.iter().copied().collect();
    let bomb_positions_ranges: HashMap<Position, usize> =
        query3.iter().map(|(p, b)| (*p, b.range)).collect();
    let fireproof_positions: HashSet<Position> = query5.iter().copied().collect();
    let wall_positions: HashSet<Position> = query7.iter().copied().collect();
    let invalid_bomb_spawn_positions: HashSet<Position> = query8.iter().copied().collect();
    let destructible_positions: HashSet<Position> = query9.iter().copied().collect();

    for (entity, position, move_cooldown, wall_hack, bomb_satchel, team_id) in query.iter() {
        let impassable_positions: HashSet<Position> = if wall_hack.is_none() {
            query4.iter().map(|(p, _)| *p).collect()
        } else {
            query4
                .iter()
                .filter_map(|(p, d)| if d.is_some() { None } else { Some(*p) })
                .collect()
        };

        // run to safety
        if move_cooldown.0.ready() {
            if let Some(safe_direction) = get_directions_to_closest_safe_positions(
                *position,
                &fire_positions,
                &bomb_positions_ranges,
                &fireproof_positions,
                &impassable_positions,
                &wall_positions,
            )
            .iter()
            .choose(&mut rng)
            {
                ev_player_action.send(PlayerActionEvent(
                    entity,
                    PlayerAction::Move(*safe_direction),
                ));
                continue;
            }
        }

        // drop a bomb if possible, if it can hit an enemy and if an escape route would exist
        let enemy_positions: Vec<Position> = query6
            .iter()
            .filter(|(_, tid)| tid.0 != team_id.0)
            .map(|(p, _)| *p)
            .collect();
        if bomb_satchel.bombs_available > 0
            && !invalid_bomb_spawn_positions.contains(position)
            && !fire_positions.contains(position)
            && bomb_can_hit_a_player(
                *position,
                bomb_satchel.bomb_range,
                &enemy_positions,
                &fireproof_positions,
                &wall_positions,
            )
        {
            let mut bomb_positions_ranges = bomb_positions_ranges.clone();
            bomb_positions_ranges.insert(*position, bomb_satchel.bomb_range);

            if !get_directions_to_closest_safe_positions(
                *position,
                &fire_positions,
                &bomb_positions_ranges,
                &fireproof_positions,
                &impassable_positions,
                &wall_positions,
            )
            .is_empty()
            {
                ev_player_action.send(PlayerActionEvent(entity, PlayerAction::DropBomb));
                continue;
            }
        }

        // chase enemies
        if move_cooldown.0.ready() {
            if let Some(safe_direction_to_enemy) =
                get_directions_to_closest_positions_with_criteria(
                    *position,
                    |position| enemy_positions.contains(&position),
                    |position| {
                        !impassable_positions.contains(&position)
                            && position_is_safe(
                                position,
                                &fire_positions,
                                &bomb_positions_ranges,
                                &fireproof_positions,
                                &wall_positions,
                            )
                    },
                )
                .iter()
                .choose(&mut rng)
            {
                ev_player_action.send(PlayerActionEvent(
                    entity,
                    PlayerAction::Move(*safe_direction_to_enemy),
                ));
                continue;
            }
        }

        // break nearby walls
        if let Some(action) = get_destructible_destroying_action(
            *position,
            bomb_satchel,
            &invalid_bomb_spawn_positions,
            &fire_positions,
            &bomb_positions_ranges,
            &fireproof_positions,
            &impassable_positions,
            &wall_positions,
            &destructible_positions,
        ) {
            ev_player_action.send(PlayerActionEvent(entity, action));
        }

        // TODO: more actions
    }
}

pub fn player_move(
    mut commands: Commands,
    mut ev_player_action: EventReader<PlayerActionEvent>,
    mut q: QuerySet<(
        QueryState<
            (
                &mut Position,
                &mut Sprite,
                Option<&WallHack>,
                Option<&BombPush>,
                Option<&mut MoveCooldown>,
            ),
            With<Player>,
        >,
        QueryState<(
            Entity,
            &Solid,
            &Position,
            Option<&Destructible>,
            Option<&Bomb>,
        )>,
    )>,
    mut query2: Query<&mut Transform>,
) {
    let solids: HashMap<Position, (Entity, bool, bool)> = q
        .q1()
        .iter()
        .map(|(e, _, p, d, b)| (*p, (e, d.is_some(), b.is_some())))
        .collect();

    for (entity, direction) in ev_player_action.iter().filter_map(|p| {
        if let PlayerAction::Move(direction) = p.1 {
            Some((p.0, direction))
        } else {
            None
        }
    }) {
        if let Ok((mut position, mut sprite, wall_hack, bomb_push, mut move_cooldown)) =
            q.q0().get_mut(entity)
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
                    commands
                        .entity(*e)
                        .insert(Moving { direction })
                        .insert(MoveCooldown(Cooldown::from_seconds(0.01)));
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
    mut q: QuerySet<(
        QueryState<(
            Entity,
            &Moving,
            &mut MoveCooldown,
            &mut Position,
            &mut Transform,
        )>,
        QueryState<&Position, Or<(With<Solid>, With<Item>, With<Player>, With<Exit>)>>,
    )>,
) {
    let impassable_positions: HashSet<Position> = q.q1().iter().copied().collect();

    for (entity, moving, mut move_cooldown, mut position, mut transform) in q.q0().iter_mut() {
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
                    commands.entity(pe).insert_bundle(ImmortalBundle::default());
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
    textures: Res<Textures>,
    fonts: Res<Fonts>,
    world_id: Res<WorldID>,
    mut ev_player_action: EventReader<PlayerActionEvent>,
    mut query: Query<(&Position, &mut BombSatchel)>,
    query2: Query<&Position, Or<(With<Solid>, With<Exit>, With<BurningItem>)>>,
) {
    for entity in ev_player_action
        .iter()
        .filter(|pa| matches!(pa.1, PlayerAction::DropBomb))
        .map(|pa| pa.0)
    {
        if let Ok((position, mut bomb_satchel)) = query.get_mut(entity) {
            if bomb_satchel.bombs_available > 0 && !query2.iter().any(|p| *p == *position) {
                println!("drop bomb: {:?}", position);
                bomb_satchel.bombs_available -= 1;

                commands
                    .spawn_bundle(SpriteBundle {
                        material: textures.bomb.clone(),
                        transform: Transform::from_xyz(get_x(position.x), get_y(position.y), 25.0),
                        sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                        ..Default::default()
                    })
                    .insert(Bomb {
                        owner: Some(entity),
                        range: bomb_satchel.bomb_range,
                    })
                    .insert(Solid)
                    .insert(Perishable {
                        timer: Timer::from_seconds(2.0, false),
                    })
                    .insert(*position)
                    .with_children(|parent| {
                        let fuse_color = COLORS[if world_id.0 == 2 { 12 } else { 14 }].into();

                        let mut text = Text::with_section(
                            '*',
                            TextStyle {
                                font: fonts.mono.clone(),
                                font_size: 2.0 * PIXEL_SCALE as f32,
                                color: fuse_color,
                            },
                            TextAlignment {
                                vertical: VerticalAlign::Center,
                                horizontal: HorizontalAlign::Center,
                            },
                        );
                        text.sections.push(TextSection {
                            value: "┐\n │".into(),
                            style: TextStyle {
                                font: fonts.mono.clone(),
                                font_size: 2.0 * PIXEL_SCALE as f32,
                                color: COLORS[0].into(),
                            },
                        });

                        parent
                            .spawn_bundle(Text2dBundle {
                                text,
                                transform: Transform::from_xyz(
                                    0.0,
                                    TILE_HEIGHT as f32 / 8.0 * 2.0,
                                    0.0,
                                ),
                                ..Default::default()
                            })
                            .insert(Fuse)
                            .insert(fuse_color)
                            .insert(Timer::from_seconds(0.1, true));
                    });
            }
        }
    }
}

pub fn animate_fuse(
    time: Res<Time>,
    fonts: Res<Fonts>,
    query: Query<&Perishable, With<Bomb>>,
    mut query2: Query<(&Parent, &mut Text, &Color, &mut Timer, &mut Transform), With<Fuse>>,
) {
    for (parent, mut text, fuse_color, mut timer, mut transform) in query2.iter_mut() {
        timer.tick(time.delta());
        let percent_left = timer.percent_left();
        let fuse_char = match percent_left {
            _ if (0.0..0.33).contains(&percent_left) => 'x',
            _ if (0.33..0.66).contains(&percent_left) => '+',
            _ if (0.66..=1.0).contains(&percent_left) => '*',
            _ => unreachable!(),
        };

        let perishable = query.get(parent.0).unwrap();
        let percent_left = perishable.timer.percent_left();

        match percent_left {
            _ if (0.66..1.0).contains(&percent_left) => {
                text.sections = vec![
                    TextSection {
                        value: fuse_char.into(),
                        style: TextStyle {
                            font: fonts.mono.clone(),
                            font_size: 2.0 * PIXEL_SCALE as f32,
                            color: *fuse_color,
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
                            color: *fuse_color,
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
                        color: *fuse_color,
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

pub fn perishable_tick(
    time: Res<Time>,
    exit_position: Option<Res<ExitPosition>>,
    mut commands: Commands,
    textures: Res<Textures>,
    game_context: Res<GameContext>,
    mut query: Query<(
        Entity,
        &mut Perishable,
        &Position,
        Option<&Bomb>,
        Option<&Wall>,
    )>,
    mut query2: Query<&mut BombSatchel>,
    mut ev_explosion: EventWriter<ExplosionEvent>,
) {
    for (entity, mut perishable, position, bomb, wall) in query.iter_mut() {
        perishable.timer.tick(time.delta());

        if perishable.timer.just_finished() {
            commands.entity(entity).despawn_recursive();

            // TODO: move into separate system
            if let Some(bomb) = bomb {
                if let Some(owner) = bomb.owner {
                    if let Ok(mut bomb_satchel) = query2.get_mut(owner) {
                        bomb_satchel.bombs_available += 1;
                    }
                }

                ev_explosion.send(ExplosionEvent(*position, bomb.range));
            }

            // TODO: move into separate system
            if wall.is_some() {
                if matches!(exit_position, Some(ref p) if p.0 == *position) {
                    commands
                        .spawn_bundle(SpriteBundle {
                            material: textures.exit.clone(),
                            transform: Transform::from_xyz(
                                get_x(position.x),
                                get_y(position.y),
                                10.0,
                            ),
                            sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                            ..Default::default()
                        })
                        .insert(*position)
                        .insert(Exit::default());
                } else if rand::thread_rng().gen_range(0.0..1.0) < ITEM_SPAWN_CHANCE {
                    generate_item_at_position(
                        *position,
                        &mut commands,
                        &textures,
                        game_context.reduced_loot,
                    );
                }
            }
        }
    }
}

pub fn handle_explosion(
    mut commands: Commands,
    textures: Res<Textures>,
    audio: Res<Audio>,
    sounds: Res<Sounds>,
    query: Query<&Position, Or<(With<Solid>, With<Exit>)>>,
    mut ev_explosion: EventReader<ExplosionEvent>,
    mut ev_burn: EventWriter<BurnEvent>,
) {
    let fireproof_positions: HashSet<Position> = query.iter().copied().collect();

    let mut sound_played = false;

    for ExplosionEvent(position, range) in ev_explosion.iter().copied() {
        if !sound_played {
            audio.stop();
            audio.play(sounds.boom.clone());
            sound_played = true;
        }

        let spawn_fire = |commands: &mut Commands, position: Position| {
            commands
                .spawn_bundle(SpriteBundle {
                    material: textures.fire.clone(),
                    transform: Transform::from_xyz(get_x(position.x), get_y(position.y), 10.0),
                    sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                    ..Default::default()
                })
                .insert(Fire)
                .insert(position)
                .insert(Perishable {
                    timer: Timer::from_seconds(0.5, false),
                });
        };

        spawn_fire(&mut commands, position);
        for direction in Direction::LIST {
            for i in 1..=range {
                let position = position.offset(direction, i);

                if fireproof_positions.contains(&position) {
                    ev_burn.send(BurnEvent(position));
                    break;
                }

                spawn_fire(&mut commands, position);
            }
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
            commands.entity(entity).remove_bundle::<ImmortalBundle>();
        }
    }
}

pub fn animate_immortality(
    time: Res<Time>,
    mut query: Query<
        (
            &Immortal,
            &mut Timer,
            &mut Handle<ColorMaterial>,
            &BaseMaterial,
            &ImmortalMaterial,
        ),
        With<Immortal>,
    >,
) {
    for (immortal, mut timer, mut color, base_material, immortal_material) in query.iter_mut() {
        if !immortal.timer.finished() {
            timer.tick(time.delta());
            let percent_left = timer.percent_left();
            match percent_left {
                _ if (0.5..=1.0).contains(&percent_left) => {
                    *color = immortal_material.0.clone();
                }
                // hackish way to end the animation contnd.
                _ => *color = base_material.0.clone(),
            };
        } else {
            *color = base_material.0.clone();
        }
    }
}

pub fn fire_effect(mut query: Query<&Position, With<Fire>>, mut ev_burn: EventWriter<BurnEvent>) {
    for position in query.iter_mut() {
        ev_burn.send(BurnEvent(*position));
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
            ev_damage.send(DamageEvent(e));
        }
    }
}

pub fn player_burn(
    query: Query<(Entity, &Position), (With<Player>, Without<Immortal>)>,
    query2: Query<&Position, With<Wall>>,
    mut ev_burn: EventReader<BurnEvent>,
    mut ev_damage: EventWriter<DamageEvent>,
) {
    for BurnEvent(position) in ev_burn.iter() {
        for (pe, player_pos) in query.iter().filter(|(_, pp)| **pp == *position) {
            if query2.iter().any(|wall_pos| *wall_pos == *player_pos) {
                // high ground, bitch
                continue;
            }

            ev_damage.send(DamageEvent(pe));
        }
    }
}

pub fn player_damage(
    mut commands: Commands,
    textures: Res<Textures>,
    map_size: Res<MapSize>,
    game_context: Res<GameContext>,
    mut game_score: Option<ResMut<GameScore>>,
    mut query: Query<
        (
            Entity,
            &mut Health,
            &mut Handle<ColorMaterial>,
            &ImmortalMaterial,
            Option<&PointValue>,
        ),
        (With<Player>, Without<Immortal>),
    >,
    query2: Query<
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
    state: Res<State<AppState>>,
    mut ev_damage: EventReader<DamageEvent>,
) {
    let mut damaged_players = HashSet::new();

    for DamageEvent(entity) in ev_damage.iter() {
        if let Ok((pe, mut health, mut color, immortal_material, point_value)) =
            query.get_mut(*entity)
        {
            if damaged_players.contains(&pe) {
                continue;
            }
            damaged_players.insert(pe);

            println!("player damaged: {:?}", pe);
            health.health -= 1;

            let mut gain_immortality = false;
            if health.health == 0 {
                health.lives -= 1;
                if health.lives == 0 {
                    println!("player died: {:?}", pe);
                    commands.entity(pe).despawn_recursive();

                    if let AppState::BattleMode = state.current() {
                        // item pinata
                        let invalid_positions: HashSet<Position> = query2.iter().copied().collect();
                        let valid_positions = (0..map_size.rows)
                            .map(|y| {
                                (0..map_size.columns).map(move |x| Position {
                                    y: y as isize,
                                    x: x as isize,
                                })
                            })
                            .flatten()
                            .filter(|p| !invalid_positions.contains(p));
                        for position in valid_positions.choose_multiple(&mut rand::thread_rng(), 3)
                        {
                            generate_item_at_position(
                                position,
                                &mut commands,
                                &textures,
                                game_context.reduced_loot,
                            );
                        }
                    }

                    if let Some(ref mut game_score) = game_score {
                        if let Some(point_value) = point_value {
                            game_score.0 += point_value.0;
                        }
                    }
                } else {
                    health.health = health.max_health;
                    gain_immortality = true;
                }
            } else {
                gain_immortality = true;
            }

            if gain_immortality {
                commands.entity(pe).insert_bundle(ImmortalBundle::default());
                *color = immortal_material.0.clone();
            }
        }
    }
}

pub fn bomb_burn(
    mut query: Query<(&mut Perishable, &Position), With<Bomb>>,
    mut ev_burn: EventReader<BurnEvent>,
) {
    for BurnEvent(position) in ev_burn.iter() {
        query
            .iter_mut()
            .filter(|(_, p)| **p == *position)
            .for_each(|(mut bp, _)| {
                const SHORTENED_FUSE_DURATION: Duration = Duration::from_millis(50);
                if bp.timer.duration() - bp.timer.elapsed() > SHORTENED_FUSE_DURATION {
                    bp.timer.set_duration(SHORTENED_FUSE_DURATION);
                    bp.timer.reset();
                }
            });
    }
}

pub fn destructible_wall_burn(
    textures: Res<Textures>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &Position,
            &mut Handle<ColorMaterial>,
            Option<&Perishable>,
        ),
        (With<Wall>, With<Destructible>),
    >,
    mut ev_burn: EventReader<BurnEvent>,
) {
    for BurnEvent(position) in ev_burn.iter() {
        for (e, _, mut c, perishable) in query.iter_mut().filter(|(_, p, _, _)| **p == *position) {
            if perishable.is_none() {
                commands.entity(e).insert(Perishable {
                    timer: Timer::from_seconds(0.5, false),
                });
                *c = textures.get_map_textures().burning_wall.clone();
            }
        }
    }
}

pub fn item_burn(
    textures: Res<Textures>,
    mut commands: Commands,
    mut query: Query<(Entity, &Position), With<Item>>,
    mut ev_burn: EventReader<BurnEvent>,
) {
    let mut burned = HashSet::new();

    for BurnEvent(position) in ev_burn.iter() {
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
            commands
                .spawn_bundle(SpriteBundle {
                    material: textures.burning_item.clone(),
                    transform: Transform::from_xyz(get_x(position.x), get_y(position.y), 20.0),
                    sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                    ..Default::default()
                })
                .insert(*position)
                .insert(BurningItem)
                .insert(Perishable {
                    timer: Timer::from_seconds(0.5, false),
                });
        }
    }
}

pub fn exit_burn(
    time: Res<Time>,
    textures: Res<Textures>,
    mut commands: Commands,
    mut query: Query<(&Position, &mut Exit)>,
    mut ev_burn: EventReader<BurnEvent>,
) {
    // we do checks here because some levels don't have exits (e.g. boss rooms)
    // TODO: make a separate state for those scenarios that don't run this system?
    if let Ok((_, mut exit)) = query.single_mut() {
        exit.spawn_cooldown.tick(time.delta());
    }

    for BurnEvent(position) in ev_burn.iter() {
        if let Ok((exit_position, mut exit)) = query.single_mut() {
            if *exit_position == *position && exit.spawn_cooldown.ready() {
                println!("exit burned: {:?}", position);

                // spawn mob
                let base_material = textures.crook.clone();
                commands
                    .spawn_bundle(SpriteBundle {
                        material: base_material.clone(),
                        transform: Transform::from_xyz(
                            get_x(exit_position.x),
                            get_y(exit_position.y),
                            50.0,
                        ),
                        sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                        ..Default::default()
                    })
                    .insert(BaseMaterial(base_material))
                    .insert(ImmortalMaterial(textures.immortal_crook.clone()))
                    .insert(Player)
                    .insert(MobAI::default())
                    .insert(MoveCooldown(Cooldown::from_seconds(0.4)))
                    .insert(Health {
                        lives: 1,
                        max_health: 1,
                        health: 1,
                    })
                    .insert(*exit_position)
                    .insert(MeleeAttacker)
                    .insert(TeamID(1))
                    .insert_bundle(ImmortalBundle::default());

                exit.spawn_cooldown.trigger();
            }
        }
    }
}

pub fn pop_state_on_enter(
    mut keyboard_input: ResMut<Input<KeyCode>>,
    mut state: ResMut<State<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Return) {
        state.pop().unwrap();
        keyboard_input.reset(KeyCode::Return);
    }
}