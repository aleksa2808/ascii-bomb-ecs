use std::collections::{HashMap, HashSet};
use std::time::Duration;

use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::render::camera::VisibleEntities;
use rand::prelude::*;
use rand::Rng;

use crate::camera::*;
use crate::components::*;
use crate::constants::*;
use crate::events::*;
use crate::helpers::*;
use crate::item::*;
use crate::resources::*;
use crate::types::{Direction, *};

pub fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // let colors = vec![
    //     (12, 12, 12),
    //     (0, 55, 218),
    //     (19, 161, 14),
    //     (58, 150, 221),
    //     (197, 15, 31),
    //     (136, 23, 152),
    //     (193, 156, 0),
    //     (204, 204, 204),
    //     (118, 118, 118),
    //     (59, 120, 255),
    //     (22, 198, 12),
    //     (97, 214, 214),
    //     (231, 72, 86),
    //     (180, 0, 158),
    //     (249, 241, 165),
    //     (242, 242, 242),
    // ];
    // for (i, c) in colors.iter().enumerate() {
    //     commands.spawn_bundle(SpriteBundle {
    //         material: materials.add(Color::rgb_u8(c.0, c.1, c.2).into()),
    //         transform: Transform::from_xyz(get_x(i as isize), get_y(-1), 50.0),
    //         sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
    //         ..Default::default()
    //     });
    // }

    let textures = Textures {
        // players + effects
        penguin: materials.add(asset_server.load("sprites/penguin.png").into()),
        immortal_penguin: materials.add(asset_server.load("sprites/immortal_penguin.png").into()),
        crook: materials.add(asset_server.load("sprites/crook.png").into()),
        immortal_crook: materials.add(asset_server.load("sprites/immortal_crook.png").into()),
        hatter: materials.add(asset_server.load("sprites/hatter.png").into()),
        immortal_hatter: materials.add(asset_server.load("sprites/immortal_hatter.png").into()),
        bat: materials.add(asset_server.load("sprites/bat.png").into()),
        immortal_bat: materials.add(asset_server.load("sprites/immortal_bat.png").into()),
        // bomb + fire
        bomb: materials.add(asset_server.load("sprites/bomb.png").into()),
        fire: materials.add(asset_server.load("sprites/fire.png").into()),
        // map tiles
        empty: materials.add(asset_server.load("sprites/empty.png").into()),
        wall: materials.add(asset_server.load("sprites/wall.png").into()),
        destructible_wall: materials.add(asset_server.load("sprites/destructible_wall.png").into()),
        burning_wall: materials.add(asset_server.load("sprites/burning_wall.png").into()),
        // items
        bombs_up: materials.add(asset_server.load("sprites/bombs_up.png").into()),
        range_up: materials.add(asset_server.load("sprites/range_up.png").into()),
        lives_up: materials.add(asset_server.load("sprites/lives_up.png").into()),
        wall_hack: materials.add(asset_server.load("sprites/wall_hack.png").into()),
        bomb_push: materials.add(asset_server.load("sprites/bomb_push.png").into()),
        immortal: materials.add(asset_server.load("sprites/immortal.png").into()),
        burning_item: materials.add(asset_server.load("sprites/burning_item.png").into()),
    };

    let fonts = Fonts {
        font1: asset_server.load("fonts/FiraMono-Medium.ttf"),
    };

    const MAP_WIDTH: usize = 15;
    const MAP_HEIGHT: usize = 11;

    // spawn camera
    let projection = SimpleOrthoProjection::new(MAP_WIDTH, MAP_HEIGHT);
    let cam_name = bevy::render::render_graph::base::camera::CAMERA_2D;
    let mut camera = Camera::default();
    camera.name = Some(cam_name.to_string());

    commands.spawn_bundle((
        Transform::from_translation(Vec3::new(0.0, 0.0, projection.far - 0.1)),
        GlobalTransform::default(),
        VisibleEntities::default(),
        camera,
        projection,
    ));

    let player_spawn_position = Position { y: 1, x: 1 };

    // map generation //

    // spawn player
    let base_material = textures.penguin.clone();
    let immortal_material = textures.immortal_penguin.clone();
    commands
        .spawn_bundle(SpriteBundle {
            material: base_material.clone(),
            transform: Transform::from_xyz(
                get_x(player_spawn_position.x),
                get_y(player_spawn_position.y),
                50.0,
            ),
            sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
            ..Default::default()
        })
        .insert(BaseMaterial(base_material))
        .insert(ImmortalMaterial(immortal_material))
        .insert(Player {})
        .insert(HumanControlled(0))
        .insert(Health {
            lives: 1,
            max_health: 2,
            health: 2,
        })
        .insert(player_spawn_position)
        .insert(BombSatchel {
            bombs_available: 3,
            bomb_range: 2,
        })
        .insert(TeamAlignment(0));

    // spawn enemies

    // TODO: move somewhere else
    let level = 1;

    let mob_num = level % 5 + 2 + level / 5;

    // hardcoded for 11x15
    let x = [
        MAP_WIDTH - 4,
        MAP_WIDTH - 2,
        11,
        5,
        1,
        MAP_WIDTH - 6,
        MAP_WIDTH - 6,
        7,
    ];
    let y = [
        MAP_HEIGHT - 8,
        1,
        MAP_HEIGHT - 2,
        MAP_HEIGHT - 6,
        9,
        5,
        MAP_HEIGHT - 4,
        7,
    ];
    let mut rng = rand::thread_rng();
    let bias = rng.gen::<usize>() % 20;

    let mut enemy_spawn_positions = vec![];
    for i in 0..mob_num {
        let (base_material, immortal_material, wall_hack, health) = if i > 3 {
            if i > 5 {
                (textures.bat.clone(), textures.immortal_bat.clone(), true, 3)
            } else {
                (
                    textures.hatter.clone(),
                    textures.immortal_hatter.clone(),
                    false,
                    2,
                )
            }
        } else {
            (
                textures.crook.clone(),
                textures.immortal_crook.clone(),
                false,
                1,
            )
        };

        let enemy_spawn_position = Position {
            x: x[(i as usize + bias) % 6] as isize,
            y: y[(i as usize + bias) % 6] as isize,
        };
        enemy_spawn_positions.push(enemy_spawn_position);

        let mut ec = commands.spawn_bundle(SpriteBundle {
            material: base_material.clone(),
            transform: Transform::from_xyz(
                get_x(enemy_spawn_position.x),
                get_y(enemy_spawn_position.y),
                50.0,
            ),
            sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
            ..Default::default()
        });
        ec.insert(BaseMaterial(base_material))
            .insert(ImmortalMaterial(immortal_material))
            .insert(Player {})
            .insert(MobAI::default())
            .insert(MoveCooldown(Timer::from_seconds(0.4, false)))
            .insert(Health {
                lives: 1,
                max_health: health,
                health,
            })
            .insert(enemy_spawn_position)
            .insert(MeleeAttacker {})
            .insert(TeamAlignment(1));

        if wall_hack {
            ec.insert(WallHack);
        }
    }

    // place empty/passable tiles
    for j in 0..MAP_HEIGHT {
        for i in 0..MAP_WIDTH {
            commands.spawn_bundle(SpriteBundle {
                material: textures.empty.clone(),
                transform: Transform::from_xyz(get_x(i as isize), get_y(j as isize), 0.0),
                sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                ..Default::default()
            });
        }
    }

    // spawn walls
    let mut stone_wall_positions = HashSet::new();
    for i in 0..MAP_WIDTH {
        // top
        stone_wall_positions.insert(Position {
            y: 0,
            x: i as isize,
        });
        // bottom
        stone_wall_positions.insert(Position {
            y: (MAP_HEIGHT - 1) as isize,
            x: i as isize,
        });
    }
    for i in 1..MAP_HEIGHT {
        // left
        stone_wall_positions.insert(Position {
            y: i as isize,
            x: 0,
        });
        // right
        stone_wall_positions.insert(Position {
            y: i as isize,
            x: (MAP_WIDTH - 1) as isize,
        });
    }
    // checkered middle
    for i in (2..MAP_HEIGHT).step_by(2) {
        for j in (2..MAP_WIDTH).step_by(2) {
            stone_wall_positions.insert(Position {
                y: i as isize,
                x: j as isize,
            });
        }
    }

    for position in stone_wall_positions.iter() {
        commands
            .spawn_bundle(SpriteBundle {
                material: textures.wall.clone(),
                transform: Transform::from_xyz(get_x(position.x), get_y(position.y), 0.0),
                sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                ..Default::default()
            })
            .insert(Wall {})
            .insert(Solid {})
            .insert(*position);
    }

    let mut destructible_wall_potential_positions: HashSet<Position> = (0..MAP_HEIGHT)
        .map(|y| {
            (0..MAP_WIDTH).map(move |x| Position {
                y: y as isize,
                x: x as isize,
            })
        })
        .flatten()
        .filter(|p| !stone_wall_positions.contains(p))
        .collect();

    let number_of_passable_positions = destructible_wall_potential_positions.len();

    // reserve room for the player (cross-shaped)
    destructible_wall_potential_positions.remove(&player_spawn_position);
    for position in Direction::LIST
        .iter()
        .map(|direction| player_spawn_position.offset(direction, 1))
    {
        destructible_wall_potential_positions.remove(&position);
    }

    // reserve room for the enemies (line-shaped)
    for enemy_spawn_position in enemy_spawn_positions {
        destructible_wall_potential_positions.remove(&enemy_spawn_position);

        for direction in [
            [Direction::Left, Direction::Right],
            [Direction::Up, Direction::Down],
        ]
        .choose(&mut rng)
        .unwrap()
        {
            for j in 1..3 {
                let position = enemy_spawn_position.offset(&direction, j);
                if stone_wall_positions.contains(&position) {
                    break;
                }
                destructible_wall_potential_positions.remove(&position);
            }
        }
    }

    let percent_of_passable_positions_to_fill = 50.0;
    let num_of_destructible_walls_to_place = (number_of_passable_positions as f32
        * percent_of_passable_positions_to_fill
        / 100.0) as usize;
    if destructible_wall_potential_positions.len() < num_of_destructible_walls_to_place {
        panic!(
            "Not enough passable positions available for placing destructible walls. Have {}, but need at least {}",
            destructible_wall_potential_positions.len(),
            num_of_destructible_walls_to_place
        );
    }

    for position in destructible_wall_potential_positions
        .iter()
        .copied()
        .choose_multiple(&mut rng, num_of_destructible_walls_to_place)
    {
        commands
            .spawn_bundle(SpriteBundle {
                material: textures.destructible_wall.clone(),
                transform: Transform::from_xyz(get_x(position.x), get_y(position.y), 0.0),
                sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                ..Default::default()
            })
            .insert(Wall {})
            .insert(Solid {})
            .insert(Destructible {})
            .insert(position);
    }

    commands.insert_resource(textures);
    commands.insert_resource(fonts);
}

pub fn handle_keyboard_input(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<(Entity, &HumanControlled), With<Player>>,
    mut ev_player_action: EventWriter<PlayerActionEvent>,
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
}

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
            let result = solids.get(&position.offset(&direction, 1));
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
            let passable_dir = potential_directions
                .iter()
                .find(|direction| {
                    let result = solids.get(&position.offset(&direction, 1));
                    result.is_none() || (wall_hack.is_some() && matches!(result, Some(true)))
                })
                .copied();
            if let Some(direction) = passable_dir {
                mob_ai.direction = passable_dir;
                ev_player_action.send(PlayerActionEvent(entity, PlayerAction::Move(direction)));
                break;
            }
        }
    }
}

pub fn player_move(
    time: Res<Time>,
    mut commands: Commands,
    mut ev_player_action: EventReader<PlayerActionEvent>,
    mut q: QuerySet<(
        Query<
            (
                Entity,
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
        if let Ok((entity, mut position, mut sprite, wall_hack, bomb_push, mut move_cooldown)) =
            q.q0_mut().get_mut(entity)
        {
            // visual / sprite flipping
            match direction {
                Direction::Left => sprite.flip_x = true,
                Direction::Right => sprite.flip_x = false,
                _ => (),
            }

            if let Some(move_cooldown) = move_cooldown.as_mut() {
                move_cooldown.0.tick(time.delta());
                if !move_cooldown.0.finished() {
                    continue;
                }
            }

            let new_position = position.offset(&direction, 1);
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
                        .insert(MoveCooldown(Timer::from_seconds(0.01, true)));
                }
            }

            if moved {
                println!("{:?}", position);

                if let Some(mut move_cooldown) = move_cooldown {
                    move_cooldown.0.reset();
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
    time: Res<Time>,
    mut commands: Commands,
    mut q: QuerySet<(
        Query<(
            Entity,
            &Moving,
            &mut MoveCooldown,
            &mut Position,
            &mut Transform,
        )>,
        Query<&Position, Or<(With<Solid>, With<Item>, With<Player>)>>,
    )>,
) {
    let impassables: HashSet<Position> = q.q1().iter().copied().collect();

    for (entity, moving, mut move_cooldown, mut position, mut transform) in q.q0_mut().iter_mut() {
        move_cooldown.0.tick(time.delta());

        if move_cooldown.0.just_finished() {
            let new_position = position.offset(&moving.direction, 1);
            if impassables.get(&new_position).is_none() {
                *position = new_position;

                let translation = &mut transform.translation;
                translation.x = get_x(position.x);
                translation.y = get_y(position.y);
            } else {
                commands.entity(entity).remove::<Moving>();
            }
        }
    }
}

pub fn pick_up_item(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Health,
            &mut Handle<ColorMaterial>,
            &Position,
            &mut BombSatchel,
            &ImmortalMaterial,
        ),
        With<Player>,
    >,
    query2: Query<(Entity, &Item, &Position)>,
) {
    let mut rng = rand::thread_rng();
    for (ie, i, ip) in query2.iter() {
        if let Some((pe, mut h, mut color, _, mut bomb_satchel, immortal_material)) = query
            .iter_mut()
            .filter(|(_, _, _, pp, _, _)| **pp == *ip)
            .choose(&mut rng)
        {
            println!("powered up: {:?}", ip);
            match i {
                Item::Upgrade(Upgrade::BombsUp) => bomb_satchel.bombs_available += 1,
                Item::Upgrade(Upgrade::RangeUp) => bomb_satchel.bomb_range += 1,
                Item::Upgrade(Upgrade::LivesUp) => h.lives += 1,
                Item::Power(Power::Immortal) => {
                    commands.entity(pe).insert_bundle(ImmortalBundle::default());
                    *color = immortal_material.0.clone();
                }
                Item::Power(Power::WallHack) => {
                    commands.entity(pe).insert(WallHack {});
                }
                Item::Power(Power::BombPush) => {
                    commands.entity(pe).insert(BombPush {});
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
    mut ev_player_action: EventReader<PlayerActionEvent>,
    mut query: Query<(&Position, &mut BombSatchel)>,
    query2: Query<&Position, With<Solid>>,
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
                        parent: entity,
                        range: bomb_satchel.bomb_range,
                    })
                    .insert(Solid {})
                    .insert(Perishable {
                        timer: Timer::from_seconds(2.0, false),
                    })
                    .insert(*position)
                    .with_children(|parent| {
                        let mut text = Text::with_section(
                            '*',
                            TextStyle {
                                font: fonts.font1.clone(),
                                font_size: 10.0,
                                color: Color::rgb_u8(249, 241, 165),
                                ..Default::default()
                            },
                            TextAlignment {
                                vertical: VerticalAlign::Center,
                                horizontal: HorizontalAlign::Center,
                            },
                        );
                        text.sections.push(TextSection {
                            value: "┐\n │".into(),
                            style: TextStyle {
                                font: fonts.font1.clone(),
                                font_size: 10.0,
                                color: Color::BLACK,
                                ..Default::default()
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
                            .insert(Fuse {})
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
    mut query2: Query<(&Parent, &mut Text, &mut Timer, &mut Transform), With<Fuse>>,
) {
    for (parent, mut text, mut timer, mut transform) in query2.iter_mut() {
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
                            font: fonts.font1.clone(),
                            font_size: 10.0,
                            color: Color::rgb_u8(249, 241, 165),
                            ..Default::default()
                        },
                    },
                    TextSection {
                        value: "┐\n │".into(),
                        style: TextStyle {
                            font: fonts.font1.clone(),
                            font_size: 10.0,
                            color: Color::BLACK,
                            ..Default::default()
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
                            font: fonts.font1.clone(),
                            font_size: 10.0,
                            color: Color::rgb_u8(249, 241, 165),
                            ..Default::default()
                        },
                    },
                    TextSection {
                        value: "\n│".into(),
                        style: TextStyle {
                            font: fonts.font1.clone(),
                            font_size: 10.0,
                            color: Color::BLACK,
                            ..Default::default()
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
                        font: fonts.font1.clone(),
                        font_size: 10.0,
                        color: Color::rgb_u8(249, 241, 165),
                        ..Default::default()
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
    mut commands: Commands,
    textures: Res<Textures>,
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

            if let Some(bomb) = bomb {
                if let Ok(mut bomb_satchel) = query2.get_mut(bomb.parent) {
                    bomb_satchel.bombs_available += 1;
                }

                ev_explosion.send(ExplosionEvent(*position, bomb.range));
            }

            if wall.is_some() {
                // generate power up
                const POWER_CHANCE: usize = 100;
                if rand::thread_rng().gen::<usize>() % 100 < POWER_CHANCE {
                    let item = Item::generate(false);
                    let mut ec = commands.spawn_bundle(SpriteBundle {
                        material: match item {
                            Item::Upgrade(Upgrade::BombsUp) => textures.bombs_up.clone(),
                            Item::Upgrade(Upgrade::RangeUp) => textures.range_up.clone(),
                            Item::Upgrade(Upgrade::LivesUp) => textures.lives_up.clone(),
                            Item::Power(Power::WallHack) => textures.wall_hack.clone(),
                            Item::Power(Power::BombPush) => textures.bomb_push.clone(),
                            Item::Power(Power::Immortal) => textures.immortal.clone(),
                        },
                        transform: Transform::from_xyz(get_x(position.x), get_y(position.y), 20.0),
                        sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                        ..Default::default()
                    });
                    ec.insert(*position).insert(item);
                }
            }
        }
    }
}

pub fn handle_explosion(
    mut commands: Commands,
    textures: Res<Textures>,
    query: Query<&Position, With<Solid>>,
    mut ev_explosion: EventReader<ExplosionEvent>,
    mut ev_burn: EventWriter<BurnEvent>,
) {
    let solids: HashSet<Position> = query.iter().copied().collect();

    for ExplosionEvent(position, range) in ev_explosion.iter().copied() {
        let spawn_fire = |commands: &mut Commands, position: Position| {
            commands
                .spawn_bundle(SpriteBundle {
                    material: textures.fire.clone(),
                    transform: Transform::from_xyz(get_x(position.x), get_y(position.y), 0.0),
                    sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                    ..Default::default()
                })
                .insert(Fire {})
                .insert(position)
                .insert(Perishable {
                    timer: Timer::from_seconds(0.5, false),
                });
        };

        spawn_fire(&mut commands, position);
        for direction in Direction::LIST {
            for i in 1..=range {
                let position = position.offset(&direction, i);

                if solids.contains(&position) {
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
    mut query: Query<(
        Entity,
        &mut Immortal,
        &mut Timer,
        &mut Handle<ColorMaterial>,
        &BaseMaterial,
    )>,
) {
    for (entity, mut immortal, mut timer, mut color, base_material) in query.iter_mut() {
        immortal.timer.tick(time.delta());

        if immortal.timer.just_finished() {
            commands.entity(entity).remove_bundle::<ImmortalBundle>();

            // hackish way to end the animation
            timer.set_duration(Duration::ZERO);
            timer.reset();
            *color = base_material.0.clone();
        }
    }
}

pub fn animate_immortality(
    time: Res<Time>,
    mut query: Query<
        (
            &mut Timer,
            &mut Handle<ColorMaterial>,
            &BaseMaterial,
            &ImmortalMaterial,
        ),
        With<Immortal>,
    >,
) {
    for (mut timer, mut color, base_material, immortal_material) in query.iter_mut() {
        timer.tick(time.delta());
        let percent_left = timer.percent_left();
        match percent_left {
            _ if (0.5..=1.0).contains(&percent_left) => {
                *color = immortal_material.0.clone();
            }
            // hackish way to end the animation contnd.
            _ => *color = base_material.0.clone(),
        };
    }
}

pub fn fire_effect(mut query: Query<&Position, With<Fire>>, mut ev_burn: EventWriter<BurnEvent>) {
    for position in query.iter_mut() {
        ev_burn.send(BurnEvent(*position));
    }
}

pub fn melee_attack(
    query: Query<(&Position, &TeamAlignment), With<MeleeAttacker>>,
    query2: Query<(Entity, &Position, &TeamAlignment), With<Player>>,
    mut ev_damage: EventWriter<DamageEvent>,
) {
    for (attacker_position, attacker_team_alignment) in query.iter() {
        for (e, _, _) in query2
            .iter()
            .filter(|(_, p, ta)| **p == *attacker_position && ta.0 != attacker_team_alignment.0)
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
    mut query: Query<
        (
            Entity,
            &mut Health,
            &mut Handle<ColorMaterial>,
            &ImmortalMaterial,
        ),
        (With<Player>, Without<Immortal>),
    >,
    mut ev_damage: EventReader<DamageEvent>,
) {
    let mut damaged_players = HashSet::new();

    for DamageEvent(entity) in ev_damage.iter() {
        if let Ok((pe, mut health, mut color, immortal_material)) = query.get_mut(*entity) {
            if damaged_players.contains(&pe) {
                continue;
            }
            damaged_players.insert(pe);

            println!("damage to player {:?}", pe);
            health.health -= 1;

            let mut gain_immortality = false;
            if health.health == 0 {
                health.lives -= 1;
                if health.lives == 0 {
                    commands.entity(pe).despawn_recursive();
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
                *c = textures.burning_wall.clone();
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
                .insert(Perishable {
                    timer: Timer::from_seconds(0.5, false),
                });
        }
    }
}
