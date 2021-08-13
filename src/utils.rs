use std::collections::HashSet;

use bevy::prelude::*;
use rand::{
    prelude::{IteratorRandom, SliceRandom},
    Rng,
};

use crate::{components::*, constants::*, resources::*, types::Direction};

pub fn get_x(x: isize) -> f32 {
    TILE_WIDTH as f32 / 2.0 + (x * TILE_WIDTH as isize) as f32
}

pub fn get_y(y: isize) -> f32 {
    -(TILE_HEIGHT as f32 / 2.0 + (y * TILE_HEIGHT as isize) as f32)
}

pub fn load_textures(
    asset_server: &AssetServer,
    materials: &mut Assets<ColorMaterial>,
    world: usize,
) -> Textures {
    Textures {
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
        empty: materials.add(
            asset_server
                .load(format!("sprites/world{}/empty.png", world).as_str())
                .into(),
        ),
        wall: materials.add(
            asset_server
                .load(format!("sprites/world{}/wall.png", world).as_str())
                .into(),
        ),
        destructible_wall: materials.add(
            asset_server
                .load(format!("sprites/world{}/destructible_wall.png", world).as_str())
                .into(),
        ),
        burning_wall: materials.add(
            asset_server
                .load(format!("sprites/world{}/burning_wall.png", world).as_str())
                .into(),
        ),
        // exit
        exit: materials.add(asset_server.load("sprites/exit.png").into()),
        // items
        bombs_up: materials.add(asset_server.load("sprites/bombs_up.png").into()),
        range_up: materials.add(asset_server.load("sprites/range_up.png").into()),
        lives_up: materials.add(asset_server.load("sprites/lives_up.png").into()),
        wall_hack: materials.add(asset_server.load("sprites/wall_hack.png").into()),
        bomb_push: materials.add(asset_server.load("sprites/bomb_push.png").into()),
        immortal: materials.add(asset_server.load("sprites/immortal.png").into()),
        burning_item: materials.add(asset_server.load("sprites/burning_item.png").into()),
    }
}

pub fn spawn_enemies(commands: &mut Commands, textures: &Textures, level: &Level) -> Vec<Position> {
    // spawn enemies
    let mob_num = level.sublevel + level.world;

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
        let (base_material, immortal_material, wall_hack, health, point_value) = if i > 3 {
            if i > 5 {
                (
                    textures.bat.clone(),
                    textures.immortal_bat.clone(),
                    true,
                    3,
                    90,
                )
            } else {
                (
                    textures.hatter.clone(),
                    textures.immortal_hatter.clone(),
                    false,
                    2,
                    70,
                )
            }
        } else {
            (
                textures.crook.clone(),
                textures.immortal_crook.clone(),
                false,
                1,
                50,
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
            .insert(TeamID(1))
            .insert(PointValue(point_value));

        if wall_hack {
            ec.insert(WallHack);
        }
    }

    enemy_spawn_positions
}

pub fn spawn_map(
    commands: &mut Commands,
    textures: &Textures,
    player_spawn_position: &Position,
    enemy_spawn_positions: &Vec<Position>,
) {
    let mut rng = rand::thread_rng();

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

    let destructible_wall_positions = destructible_wall_potential_positions
        .into_iter()
        .choose_multiple(&mut rng, num_of_destructible_walls_to_place);
    for position in &destructible_wall_positions {
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
            .insert(*position);
    }

    if let Some(position) = destructible_wall_positions.choose(&mut rng) {
        commands.insert_resource(ExitPosition(*position));
    }
}
