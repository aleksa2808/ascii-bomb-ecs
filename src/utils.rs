use std::collections::{HashMap, HashSet, VecDeque};

use bevy::prelude::*;
use rand::{
    prelude::{IteratorRandom, SliceRandom},
    Rng,
};

use crate::{components::*, constants::*, resources::*, types::Direction, types::*};

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
    // spawn mobs
    let mob_num = if let SubLevel::Regular(num) = level.sublevel {
        num + 1
    } else {
        1
    } + level.world;

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

    if let SubLevel::BossRoom = level.sublevel {
        // spawn boss
        let boss_spawn_position = Position {
            y: 3,
            x: MAP_WIDTH as isize / 2,
        };
        enemy_spawn_positions.push(boss_spawn_position);
        let base_material = textures.penguin.clone();
        let immortal_material = textures.immortal_penguin.clone();
        commands
            .spawn_bundle(SpriteBundle {
                material: base_material.clone(),
                transform: Transform::from_xyz(
                    get_x(boss_spawn_position.x),
                    get_y(boss_spawn_position.y),
                    50.0,
                ),
                sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                ..Default::default()
            })
            .insert(BaseMaterial(base_material))
            .insert(ImmortalMaterial(immortal_material))
            .insert(Player {})
            .insert(BotAI)
            .insert(MoveCooldown(Timer::from_seconds(0.4, false)))
            .insert(Health {
                lives: 1,
                max_health: 2,
                health: 2,
            })
            .insert(boss_spawn_position)
            .insert(BombSatchel {
                bombs_available: 1 + level.world,
                bomb_range: 1 + level.world,
            })
            .insert(TeamID(1))
            .insert(PointValue(200));
    }

    enemy_spawn_positions
}

pub fn spawn_map(
    commands: &mut Commands,
    textures: &Textures,
    player_spawn_position: &Position,
    enemy_spawn_positions: &[Position],
    level: &Level,
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
    destructible_wall_potential_positions.remove(player_spawn_position);
    for position in Direction::LIST
        .iter()
        .map(|direction| player_spawn_position.offset(direction, 1))
    {
        destructible_wall_potential_positions.remove(&position);
    }

    // reserve room for the enemies (line-shaped)
    for enemy_spawn_position in enemy_spawn_positions {
        destructible_wall_potential_positions.remove(enemy_spawn_position);

        for direction in [
            [Direction::Left, Direction::Right],
            [Direction::Up, Direction::Down],
        ]
        .choose(&mut rng)
        .unwrap()
        {
            for j in 1..3 {
                let position = enemy_spawn_position.offset(direction, j);
                if stone_wall_positions.contains(&position) {
                    break;
                }
                destructible_wall_potential_positions.remove(&position);
            }
        }
    }

    let percent_of_passable_positions_to_fill = if let SubLevel::BossRoom = level.sublevel {
        0.0
    } else {
        50.0
    };
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

    if let SubLevel::Regular(_) = level.sublevel {
        commands.insert_resource(ExitPosition(
            *destructible_wall_positions.choose(&mut rng).unwrap(),
        ));
    }
}

fn bomb_explosions_can_reach_position(
    bomb_positions_ranges: &HashMap<Position, usize>,
    target_position: Position,
    fireproof_positions: &HashSet<Position>,
) -> bool {
    let dangerous_bomb_positions = bomb_positions_ranges
        .iter()
        .filter(|(bomb_position, bomb_range)| {
            (target_position.y == bomb_position.y
                && (target_position.x - bomb_position.x).abs() as usize <= **bomb_range)
                || (target_position.x == bomb_position.x
                    && (target_position.y - bomb_position.y).abs() as usize <= **bomb_range)
        })
        .map(|(p, _)| *p);

    // if the position is protected from the closest bomb in each direction, than it's protected from all the others as well
    let mut closest_dangerous_bombs: HashMap<Direction, usize> = HashMap::new();
    for bomb_position in dangerous_bomb_positions {
        if target_position == bomb_position {
            // there's a bomb right on the position
            return true;
        }

        let (distance_to_bomb, direction) = if target_position.x != bomb_position.x {
            (
                (target_position.x - bomb_position.x).abs() as usize,
                if target_position.x < bomb_position.x {
                    Direction::Right
                } else {
                    Direction::Left
                },
            )
        } else {
            (
                (target_position.y - bomb_position.y).abs() as usize,
                if target_position.y < bomb_position.y {
                    Direction::Down
                } else {
                    Direction::Up
                },
            )
        };

        if let Some(min) = closest_dangerous_bombs.get_mut(&direction) {
            if distance_to_bomb < *min {
                *min = distance_to_bomb;
            }
        } else {
            closest_dangerous_bombs.insert(direction, distance_to_bomb);
        }
    }

    closest_dangerous_bombs.iter().any(|(direction, distance)| {
        !(1..*distance)
            .map(|i| target_position.offset(direction, i))
            .any(|p| fireproof_positions.contains(&p))
    })
}

pub fn position_is_safe(
    position: Position,
    fire_positions: &HashSet<Position>,
    bomb_positions_ranges: &HashMap<Position, usize>,
    fireproof_positions: &HashSet<Position>,
    wall_positions: &HashSet<Position>,
) -> bool {
    // TODO: port over additional safety checks

    // standing on a wall protects from fire
    if wall_positions.contains(&position) {
        return true;
    }

    // not standing in fire
    if fire_positions.contains(&position) {
        return false;
    }

    // not in range of bombs
    !bomb_explosions_can_reach_position(bomb_positions_ranges, position, fireproof_positions)
}

pub fn get_directions_to_closest_safe_positions(
    starting_position: Position,
    fire_positions: &HashSet<Position>,
    bomb_positions_ranges: &HashMap<Position, usize>,
    fireproof_positions: &HashSet<Position>,
    impassable_positions: &HashSet<Position>,
    wall_positions: &HashSet<Position>,
) -> HashSet<Direction> {
    // BFS lookup
    let mut result: Option<(HashSet<Direction>, usize)> = None;

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_front((starting_position, vec![]));
    while let Some((current_position, current_path)) = queue.pop_back() {
        if let Some((_, result_path_len)) = result {
            if current_path.len() > result_path_len {
                break;
            }
        }

        if visited.contains(&current_position) {
            continue;
        }
        visited.insert(current_position);

        if position_is_safe(
            current_position,
            fire_positions,
            bomb_positions_ranges,
            fireproof_positions,
            wall_positions,
        ) {
            let (result_set, _) = result.get_or_insert((HashSet::new(), current_path.len()));
            if let Some(direction) = current_path.first().copied() {
                result_set.insert(direction);
            }
            continue;
        }

        for direction in Direction::LIST {
            if matches!(current_path.last(), Some(d) if *d == direction.opposite()) {
                continue;
            }

            let adjacent_position = current_position.offset(&direction, 1);
            if !impassable_positions.contains(&adjacent_position) {
                let mut extended_path = current_path.clone();
                extended_path.push(direction);
                queue.push_front((adjacent_position, extended_path));
            }
        }
    }

    if let Some((result_set, _)) = result {
        result_set
    } else {
        HashSet::new()
    }
}

pub fn bomb_can_hit_a_player(
    bomb_position: Position,
    bomb_range: usize,
    player_positions: &[Position],
    fireproof_positions: &HashSet<Position>,
    wall_positions: &HashSet<Position>,
) -> bool {
    // questionable price for code reusability
    let bomb_positions_ranges = HashMap::from([(bomb_position, bomb_range)]);

    player_positions.iter().any(|player_position| {
        !wall_positions.contains(player_position)
            && bomb_explosions_can_reach_position(
                &bomb_positions_ranges,
                *player_position,
                fireproof_positions,
            )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // .....
    // .....
    // ..P..
    // .....
    // .....
    #[test]
    fn test_is_position_safe_no_danger() {
        let fire_positions = HashSet::new();
        let bomb_positions_ranges = HashMap::new();
        let fireproof_positions = HashSet::new();
        let wall_positions = HashSet::new();

        assert!(position_is_safe(
            Position { y: 2, x: 2 },
            &fire_positions,
            &bomb_positions_ranges,
            &fireproof_positions,
            &wall_positions,
        ));
    }

    // .....
    // .....
    // ..p..
    // .....
    // .....
    #[test]
    fn test_is_position_safe_on_top_of_bomb() {
        let fire_positions = HashSet::new();
        let bomb_positions_ranges = HashMap::from([(Position { y: 2, x: 2 }, 1)]);
        let fireproof_positions = bomb_positions_ranges.iter().map(|(p, _)| *p).collect();
        let wall_positions = HashSet::new();

        assert!(!position_is_safe(
            Position { y: 2, x: 2 },
            &fire_positions,
            &bomb_positions_ranges,
            &fireproof_positions,
            &wall_positions,
        ));
    }

    // .....
    // ..P..
    // ..f..
    // ..B..
    // .....
    #[test]
    fn test_is_position_safe_blocked_bomb() {
        let fire_positions = HashSet::new();
        let bomb_positions_ranges = HashMap::from([(Position { y: 7, x: 2 }, 2)]);
        let fireproof_positions = bomb_positions_ranges
            .iter()
            .map(|(p, _)| *p)
            .chain([Position { y: 6, x: 2 }])
            .collect();
        let wall_positions = HashSet::new();

        assert!(position_is_safe(
            Position { y: 1, x: 2 },
            &fire_positions,
            &bomb_positions_ranges,
            &fireproof_positions,
            &wall_positions,
        ));
    }

    // .....
    // ..P..
    // .....
    // ..B..
    // .....
    #[test]
    fn test_is_position_safe_out_of_range_bomb() {
        let fire_positions = HashSet::new();
        let bomb_positions_ranges = HashMap::from([(Position { y: 3, x: 2 }, 1)]);
        let fireproof_positions = bomb_positions_ranges.iter().map(|(p, _)| *p).collect();
        let wall_positions = HashSet::new();

        assert!(position_is_safe(
            Position { y: 1, x: 2 },
            &fire_positions,
            &bomb_positions_ranges,
            &fireproof_positions,
            &wall_positions,
        ));
    }

    // .....
    // ..P..
    // .....
    // ..B..
    // .....
    #[test]
    fn test_is_position_safe_in_danger() {
        let fire_positions = HashSet::new();
        let bomb_positions_ranges = HashMap::from([(Position { y: 3, x: 2 }, 2)]);
        let fireproof_positions = bomb_positions_ranges.iter().map(|(p, _)| *p).collect();
        let wall_positions = HashSet::new();

        assert!(!position_is_safe(
            Position { y: 1, x: 2 },
            &fire_positions,
            &bomb_positions_ranges,
            &fireproof_positions,
            &wall_positions,
        ));
    }

    // .....
    // .IPI.
    // ..B..
    // .....
    #[test]
    fn test_is_position_safe_in_danger_blocked_path() {
        let fire_positions = HashSet::new();
        let bomb_positions_ranges = HashMap::from([(Position { y: 2, x: 2 }, 1)]);
        let fireproof_positions = bomb_positions_ranges
            .iter()
            .map(|(p, _)| *p)
            .chain([Position { y: 1, x: 1 }, Position { y: 1, x: 3 }])
            .collect();
        let wall_positions = HashSet::new();

        assert!(!position_is_safe(
            Position { y: 1, x: 2 },
            &fire_positions,
            &bomb_positions_ranges,
            &fireproof_positions,
            &wall_positions,
        ));
    }

    // ......
    // ..I.I.
    // .BP...
    // ..IBI.
    // ......
    #[test]
    fn test_is_position_safe_in_danger_custom() {
        let fire_positions = HashSet::new();
        let bomb_positions_ranges =
            HashMap::from([(Position { y: 2, x: 1 }, 2), (Position { y: 3, x: 3 }, 2)]);
        let fireproof_positions = bomb_positions_ranges
            .iter()
            .map(|(p, _)| *p)
            .chain([
                Position { y: 1, x: 2 },
                Position { y: 3, x: 2 },
                Position { y: 1, x: 4 },
                Position { y: 3, x: 4 },
            ])
            .collect();
        let wall_positions = HashSet::new();

        assert!(!position_is_safe(
            Position { y: 2, x: 2 },
            &fire_positions,
            &bomb_positions_ranges,
            &fireproof_positions,
            &wall_positions,
        ));
    }

    // .....
    // .....
    // ..P..
    // .....
    // .....
    #[test]
    fn test_get_directions_to_closest_safe_positions_no_danger() {
        let fire_positions = HashSet::new();
        let bomb_positions_ranges = HashMap::new();
        let fireproof_positions = HashSet::new();
        let impassable_positions = HashSet::new();
        let wall_positions = HashSet::new();

        let dirs = get_directions_to_closest_safe_positions(
            Position { y: 2, x: 2 },
            &fire_positions,
            &bomb_positions_ranges,
            &fireproof_positions,
            &impassable_positions,
            &wall_positions,
        );

        assert!(dirs.is_empty());
    }

    // .....
    // ..P..
    // ..f..
    // ..B..
    // .....
    #[test]
    fn test_get_directions_to_closest_safe_positions_blocked_bomb() {
        let fire_positions = HashSet::new();
        let bomb_positions_ranges = HashMap::from([(Position { y: 7, x: 2 }, 2)]);
        let fireproof_positions = bomb_positions_ranges
            .iter()
            .map(|(p, _)| *p)
            .chain([Position { y: 6, x: 2 }])
            .collect();
        let impassable_positions = bomb_positions_ranges.iter().map(|(p, _)| *p).collect();
        let wall_positions = HashSet::new();

        let dirs = get_directions_to_closest_safe_positions(
            Position { y: 1, x: 2 },
            &fire_positions,
            &bomb_positions_ranges,
            &fireproof_positions,
            &impassable_positions,
            &wall_positions,
        );

        assert!(dirs.is_empty());
    }

    // .....
    // ..P..
    // .....
    // ..B..
    // .....
    #[test]
    fn test_get_directions_to_closest_safe_positions_out_of_range_bomb() {
        let fire_positions = HashSet::new();
        let bomb_positions_ranges = HashMap::from([(Position { y: 3, x: 2 }, 1)]);
        let fireproof_positions = bomb_positions_ranges.iter().map(|(p, _)| *p).collect();
        let impassable_positions = bomb_positions_ranges.iter().map(|(p, _)| *p).collect();
        let wall_positions = HashSet::new();

        let dirs = get_directions_to_closest_safe_positions(
            Position { y: 1, x: 2 },
            &fire_positions,
            &bomb_positions_ranges,
            &fireproof_positions,
            &impassable_positions,
            &wall_positions,
        );

        assert!(dirs.is_empty());
    }

    // .....
    // ..P..
    // .....
    // ..B..
    // .....
    #[test]
    fn test_get_directions_to_closest_safe_positions_in_danger() {
        let fire_positions = HashSet::new();
        let bomb_positions_ranges = HashMap::from([(Position { y: 3, x: 2 }, 2)]);
        let fireproof_positions = bomb_positions_ranges.iter().map(|(p, _)| *p).collect();
        let impassable_positions = bomb_positions_ranges.iter().map(|(p, _)| *p).collect();
        let wall_positions = HashSet::new();

        let dirs = get_directions_to_closest_safe_positions(
            Position { y: 1, x: 2 },
            &fire_positions,
            &bomb_positions_ranges,
            &fireproof_positions,
            &impassable_positions,
            &wall_positions,
        );

        assert_eq!(dirs.len(), 3);
        assert!(dirs.contains(&Direction::Left));
        assert!(dirs.contains(&Direction::Right));
        assert!(dirs.contains(&Direction::Up));
    }

    // .....
    // .IPI.
    // ..B..
    // .....
    #[test]
    fn test_get_directions_to_closest_safe_positions_in_danger_blocked_path() {
        let fire_positions = HashSet::new();
        let bomb_positions_ranges = HashMap::from([(Position { y: 2, x: 2 }, 1)]);
        let fireproof_positions = bomb_positions_ranges
            .iter()
            .map(|(p, _)| *p)
            .chain([Position { y: 1, x: 1 }, Position { y: 1, x: 3 }])
            .collect();
        let impassable_positions = bomb_positions_ranges
            .iter()
            .map(|(p, _)| *p)
            .chain([Position { y: 1, x: 1 }, Position { y: 1, x: 3 }])
            .collect();
        let wall_positions = HashSet::new();

        let dirs = get_directions_to_closest_safe_positions(
            Position { y: 1, x: 2 },
            &fire_positions,
            &bomb_positions_ranges,
            &fireproof_positions,
            &impassable_positions,
            &wall_positions,
        );

        assert_eq!(dirs.len(), 1);
        assert!(dirs.contains(&Direction::Up));
    }

    // ......
    // ..I.I.
    // .BP...
    // ..IBI.
    // ......
    #[test]
    fn test_get_directions_to_closest_safe_positions_in_danger_custom() {
        let fire_positions = HashSet::new();
        let bomb_positions_ranges =
            HashMap::from([(Position { y: 2, x: 1 }, 2), (Position { y: 3, x: 3 }, 2)]);
        let fireproof_positions = bomb_positions_ranges
            .iter()
            .map(|(p, _)| *p)
            .chain([
                Position { y: 1, x: 2 },
                Position { y: 3, x: 2 },
                Position { y: 1, x: 4 },
                Position { y: 3, x: 4 },
            ])
            .collect();
        let impassable_positions = bomb_positions_ranges
            .iter()
            .map(|(p, _)| *p)
            .chain([
                Position { y: 1, x: 2 },
                Position { y: 3, x: 2 },
                Position { y: 1, x: 4 },
                Position { y: 3, x: 4 },
            ])
            .collect();
        let wall_positions = HashSet::new();

        let dirs = get_directions_to_closest_safe_positions(
            Position { y: 2, x: 2 },
            &fire_positions,
            &bomb_positions_ranges,
            &fireproof_positions,
            &impassable_positions,
            &wall_positions,
        );

        assert_eq!(dirs.len(), 1);
        assert!(dirs.contains(&Direction::Right));
    }

    // .....
    // ..I..
    // .IPI.
    // ..B..
    // .....
    #[test]
    fn test_get_directions_to_closest_safe_positions_in_danger_no_exit() {
        let fire_positions = HashSet::new();
        let bomb_positions_ranges = HashMap::from([(Position { y: 3, x: 2 }, 1)]);
        let fireproof_positions = bomb_positions_ranges
            .iter()
            .map(|(p, _)| *p)
            .chain([
                Position { y: 1, x: 2 },
                Position { y: 2, x: 1 },
                Position { y: 2, x: 3 },
            ])
            .collect();
        let impassable_positions = bomb_positions_ranges
            .iter()
            .map(|(p, _)| *p)
            .chain([
                Position { y: 1, x: 2 },
                Position { y: 2, x: 1 },
                Position { y: 2, x: 3 },
            ])
            .collect();
        let wall_positions = HashSet::new();

        let dirs = get_directions_to_closest_safe_positions(
            Position { y: 2, x: 2 },
            &fire_positions,
            &bomb_positions_ranges,
            &fireproof_positions,
            &impassable_positions,
            &wall_positions,
        );

        assert!(dirs.is_empty());
    }

    // .....
    // ..P..
    // ..f..
    // ..B..
    // .....
    #[test]
    fn test_bomb_can_hit_a_player_blocked_bomb() {
        let bomb_positions_ranges = HashMap::from([(Position { y: 7, x: 2 }, 2)]);
        let fireproof_positions = bomb_positions_ranges
            .iter()
            .map(|(p, _)| *p)
            .chain([Position { y: 6, x: 2 }])
            .collect();
        let wall_positions = HashSet::new();

        assert!(!bomb_can_hit_a_player(
            Position { y: 7, x: 2 },
            2,
            &[Position { y: 1, x: 2 }],
            &fireproof_positions,
            &wall_positions,
        ));
    }

    // .....
    // .....
    // ..p..
    // .....
    // .....
    #[test]
    fn test_bomb_can_hit_a_player_on_top_of_bomb() {
        let bomb_position = Position { y: 2, x: 2 };
        let fireproof_positions = HashSet::from([bomb_position]);
        let wall_positions = HashSet::new();

        assert!(bomb_can_hit_a_player(
            bomb_position,
            2,
            &[Position { y: 2, x: 2 }],
            &fireproof_positions,
            &wall_positions,
        ));
    }

    // .....
    // ..P..
    // .....
    // ..B..
    // .....
    #[test]
    fn test_bomb_can_hit_a_player_in_danger() {
        let bomb_position = Position { y: 3, x: 2 };
        let fireproof_positions = HashSet::from([bomb_position]);
        let wall_positions = HashSet::new();

        assert!(bomb_can_hit_a_player(
            bomb_position,
            2,
            &[Position { y: 1, x: 2 }],
            &fireproof_positions,
            &wall_positions,
        ));
    }
}
