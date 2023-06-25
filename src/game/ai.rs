use bevy::{prelude::Entity, utils::HashSet};
use rand::{prelude::SliceRandom, Rng};

use super::{
    components::{BombSatchel, Position},
    resources::{MapSize, WallOfDeath},
    types::{Direction, PlayerAction},
};

pub fn safe(
    position: Position,
    fire_positions: &HashSet<Position>,
    bomb_positions: &HashSet<Position>,
    assumed_bomb_range: usize,
    fireproof_positions: &HashSet<Position>,
    wall_of_death: Option<&WallOfDeath>,
    map_size: MapSize,
) -> bool {
    // not standing in fire
    if fire_positions.contains(&position) {
        return false;
    }

    // not standing in the wall-of-death path, if it's present
    if let Some(w) = wall_of_death {
        let in_danger = match w {
            WallOfDeath::Dormant(t) => {
                t.percent_left() * t.duration().as_secs_f32() < 5.0 && position.x == 1
            }
            WallOfDeath::Active(w) => match w.direction {
                Direction::Left => {
                    // avoid bottom and left edge
                    position.y == w.position.y
                        || (map_size.rows as isize - 1 - w.position.y == 1 && position.x == 2)
                }
                Direction::Right => {
                    // avoid top and right edge
                    position.y == w.position.y
                        || position.x == map_size.columns as isize - 1 - w.position.y
                }
                Direction::Up => {
                    // avoid left and top edge
                    position.x == w.position.x || position.y == w.position.x
                }
                Direction::Down => {
                    // avoid right and bottom edge
                    position.x == w.position.x
                        || position.y
                            == map_size.rows as isize
                                - 2
                                - (map_size.columns as isize - 2 - w.position.x)
                }
            },
            _ => false,
        };

        if in_danger {
            return false;
        }
    }

    // not in range of bombs
    if bomb_positions.contains(&position) {
        return false;
    }
    for direction in Direction::LIST {
        for i in 1..=assumed_bomb_range {
            let position = position.offset(direction, i);
            if bomb_positions.contains(&position) {
                return false;
            }
            if fireproof_positions.contains(&position) {
                break;
            }
        }
    }

    true
}

pub fn safe_dir(
    starting_position: Position,
    fire_positions: &HashSet<Position>,
    bomb_positions: &HashSet<Position>,
    assumed_bomb_range: usize,
    fireproof_positions: &HashSet<Position>,
    impassable_positions: &HashSet<Position>,
    wall_of_death: Option<&WallOfDeath>,
    map_size: MapSize,
    can_push_bombs: bool,
    moving_object_stoppers: &HashSet<Position>,
) -> HashSet<Direction> {
    let mut result = HashSet::default();

    let safe = |position| {
        safe(
            position,
            fire_positions,
            bomb_positions,
            assumed_bomb_range,
            fireproof_positions,
            wall_of_death,
            map_size,
        )
    };
    let mut min = 5;
    for direction in Direction::LIST {
        let range = min;
        for i in 1..=range {
            let position = starting_position.offset(direction, i);
            if impassable_positions.contains(&position) || fire_positions.contains(&position) {
                break;
            } else {
                let side_directions = match direction {
                    Direction::Left | Direction::Right => (Direction::Up, Direction::Down),
                    Direction::Up | Direction::Down => (Direction::Left, Direction::Right),
                };
                let side_positions = (
                    position.offset(side_directions.0, 1),
                    position.offset(side_directions.1, 1),
                );

                if safe(position)
                    || (!impassable_positions.contains(&side_positions.0) && safe(side_positions.0))
                    || (!impassable_positions.contains(&side_positions.1) && safe(side_positions.1))
                {
                    if i < min {
                        result.clear();
                        min = i;
                    }
                    result.insert(direction);
                    break;
                }
            }
        }
    }

    if result.is_empty() && can_push_bombs {
        min = 5;
        for direction in Direction::LIST {
            let range = min;
            for i in 1..=range {
                let position = starting_position.offset(direction, i);
                if (impassable_positions.contains(&position) && !bomb_positions.contains(&position))
                    || fire_positions.contains(&position)
                {
                    break;
                } else if bomb_positions.contains(&position)
                    && !moving_object_stoppers.contains(&position.offset(direction, 1))
                {
                    if i < min {
                        result.clear();
                        min = i;
                    }
                    result.insert(direction);
                    break;
                }
            }
        }
    }

    result
}

pub fn detect_powers(
    starting_position: Position,
    impassable_positions: &HashSet<Position>,
    fire_positions: &HashSet<Position>,
    bomb_positions: &HashSet<Position>,
    assumed_bomb_range: usize,
    fireproof_positions: &HashSet<Position>,
    wall_of_death: Option<&WallOfDeath>,
    map_size: MapSize,
    item_positions: &HashSet<Position>,
) -> HashSet<Direction> {
    let mut result = HashSet::default();

    let safe = |position| {
        safe(
            position,
            fire_positions,
            bomb_positions,
            assumed_bomb_range,
            fireproof_positions,
            wall_of_death,
            map_size,
        )
    };
    let mut min = 5;
    for direction in Direction::LIST {
        let range = min;
        for i in 1..range {
            let position = starting_position.offset(direction, i);

            if impassable_positions.contains(&position) || !safe(position) {
                break;
            } else {
                let side_directions = match direction {
                    Direction::Left | Direction::Right => (Direction::Up, Direction::Down),
                    Direction::Up | Direction::Down => (Direction::Left, Direction::Right),
                };
                let side_positions = (
                    position.offset(side_directions.0, 1),
                    position.offset(side_directions.1, 1),
                );

                if item_positions.contains(&position)
                    || (!impassable_positions.contains(&side_positions.0)
                        && safe(side_positions.0)
                        && item_positions.contains(&side_positions.0))
                    || (!impassable_positions.contains(&side_positions.1)
                        && safe(side_positions.1)
                        && item_positions.contains(&side_positions.1))
                {
                    if i < min {
                        result.clear();
                        min = i;
                    }
                    result.insert(direction);
                    break;
                }
            }
        }
    }

    result
}

pub fn can_kill(
    bomb_position: Position,
    bomb_range: usize,
    enemy_positions: &[Position],
    stone_wall_positions: &HashSet<Position>,
) -> bool {
    for position in enemy_positions {
        if (position.y == bomb_position.y
            && (position.x - bomb_position.x).unsigned_abs() <= bomb_range
            && !(stone_wall_positions.contains(&position.offset(Direction::Right, 1))
                && stone_wall_positions.contains(&position.offset(Direction::Left, 1))))
            || (position.x == bomb_position.x
                && (position.y - bomb_position.y).unsigned_abs() <= bomb_range
                && !(stone_wall_positions.contains(&position.offset(Direction::Down, 1))
                    && stone_wall_positions.contains(&position.offset(Direction::Up, 1))))
        {
            return true;
        }
    }

    false
}

pub fn players_in_range(position: Position, player_positions: &[Position], range: usize) -> bool {
    for player_position in player_positions {
        if (player_position.y - position.y).unsigned_abs() <= range
            && (player_position.x - position.x).unsigned_abs() <= range
        {
            return true;
        }
    }

    false
}

fn damage_made(
    bomb_position: Position,
    bomb_range: usize,
    fireproof_positions: &HashSet<Position>,
    destructible_positions: &HashSet<Position>,
) -> usize {
    let mut destructibles_in_range = 0;

    for direction in Direction::LIST {
        for i in 1..=bomb_range {
            let position = bomb_position.offset(direction, i);
            if destructible_positions.contains(&position) {
                destructibles_in_range += 1;
            }
            if fireproof_positions.contains(&position) {
                break;
            }
        }
    }

    destructibles_in_range
}

fn can_place_bomb(position: Position, invalid_bomb_spawn_positions: &HashSet<Position>) -> bool {
    !invalid_bomb_spawn_positions.contains(&position)
}

pub fn should_place_bomb(
    position: Position,
    invalid_bomb_spawn_positions: &HashSet<Position>,
    bomb_positions: &HashSet<Position>,
    assumed_bomb_range: usize,
    fire_positions: &HashSet<Position>,
    fireproof_positions: &HashSet<Position>,
    impassable_positions: &HashSet<Position>,
    wall_of_death: Option<&WallOfDeath>,
    map_size: MapSize,
    can_push_bombs: bool,
    moving_object_stoppers: &HashSet<Position>,
) -> bool {
    if can_place_bomb(position, invalid_bomb_spawn_positions) {
        let mut bomb_positions = bomb_positions.clone();
        bomb_positions.insert(position);

        if !safe_dir(
            position,
            fire_positions,
            &bomb_positions,
            assumed_bomb_range,
            fireproof_positions,
            impassable_positions,
            wall_of_death,
            map_size,
            can_push_bombs,
            moving_object_stoppers,
        )
        .is_empty()
        {
            return true;
        }
    }

    false
}

pub fn destroy_blocks(
    position: Position,
    bomb_satchel: &BombSatchel,
    invalid_bomb_spawn_positions: &HashSet<Position>,
    fire_positions: &HashSet<Position>,
    bomb_positions: &HashSet<Position>,
    assumed_bomb_range: usize,
    fireproof_positions: &HashSet<Position>,
    impassable_positions: &HashSet<Position>,
    destructible_positions: &HashSet<Position>,
    wall_of_death: Option<&WallOfDeath>,
    map_size: MapSize,
    can_push_bombs: bool,
    moving_object_stoppers: &HashSet<Position>,
) -> Option<PlayerAction> {
    let mut action = None;
    let mut max_destruction_potential = 0;

    if bomb_satchel.bombs_available > 0
        && should_place_bomb(
            position,
            invalid_bomb_spawn_positions,
            bomb_positions,
            assumed_bomb_range,
            fire_positions,
            fireproof_positions,
            impassable_positions,
            wall_of_death,
            map_size,
            can_push_bombs,
            moving_object_stoppers,
        )
    {
        max_destruction_potential = damage_made(
            position,
            bomb_satchel.bomb_range,
            fireproof_positions,
            destructible_positions,
        );
        if max_destruction_potential > 0 {
            action = Some(PlayerAction::DropBomb);
        }
    }

    let mut directions: Vec<Direction> = Direction::LIST.into();
    directions.shuffle(&mut rand::thread_rng());
    for direction in directions {
        let position = position.offset(direction, 1);
        if !impassable_positions.contains(&position)
            && safe(
                position,
                fire_positions,
                bomb_positions,
                assumed_bomb_range,
                fireproof_positions,
                wall_of_death,
                map_size,
            )
            && should_place_bomb(
                position,
                invalid_bomb_spawn_positions,
                bomb_positions,
                assumed_bomb_range,
                fire_positions,
                fireproof_positions,
                impassable_positions,
                wall_of_death,
                map_size,
                can_push_bombs,
                moving_object_stoppers,
            )
        {
            let destruction_potential = damage_made(
                position,
                bomb_satchel.bomb_range,
                fireproof_positions,
                destructible_positions,
            );
            if destruction_potential > max_destruction_potential {
                max_destruction_potential = destruction_potential;
                action = Some(PlayerAction::Move(direction));
            }
        }
    }

    action
}

pub fn flee(
    starting_position: Position,
    enemy_positions: &[Position],
    impassable_positions: &HashSet<Position>,
    fire_positions: &HashSet<Position>,
    bomb_positions: &HashSet<Position>,
    assumed_bomb_range: usize,
    fireproof_positions: &HashSet<Position>,
    wall_of_death: Option<&WallOfDeath>,
    map_size: MapSize,
) -> HashSet<Direction> {
    const ENEMY_AVOIDANCE_RANGE: usize = 2;

    let mut result = HashSet::default();

    let safe = |position| {
        safe(
            position,
            fire_positions,
            bomb_positions,
            assumed_bomb_range,
            fireproof_positions,
            wall_of_death,
            map_size,
        )
    };
    let mut min = 5;
    if players_in_range(starting_position, enemy_positions, ENEMY_AVOIDANCE_RANGE) {
        for direction in Direction::LIST {
            let range = min;
            for i in 1..range {
                let position = starting_position.offset(direction, i);
                if impassable_positions.contains(&position) || !safe(position) {
                    break;
                } else {
                    let side_directions = match direction {
                        Direction::Left | Direction::Right => (Direction::Up, Direction::Down),
                        Direction::Up | Direction::Down => (Direction::Left, Direction::Right),
                    };
                    let side_positions = (
                        position.offset(side_directions.0, 1),
                        position.offset(side_directions.1, 1),
                    );

                    if players_in_range(position, enemy_positions, ENEMY_AVOIDANCE_RANGE)
                        || (!impassable_positions.contains(&side_positions.0)
                            && safe(side_positions.0)
                            && !players_in_range(
                                side_positions.0,
                                enemy_positions,
                                ENEMY_AVOIDANCE_RANGE,
                            ))
                        || (!impassable_positions.contains(&side_positions.1)
                            && safe(side_positions.1)
                            && !players_in_range(
                                side_positions.1,
                                enemy_positions,
                                ENEMY_AVOIDANCE_RANGE,
                            ))
                    {
                        if i < min {
                            result.clear();
                            min = i;
                        }
                        result.insert(direction);
                        break;
                    }
                }
            }
        }
    }

    result
}

fn dist(p1: Position, p2: Position) -> f32 {
    f32::sqrt((isize::pow(p1.y - p2.y, 2) + isize::pow(p1.x - p2.x, 2)) as f32)
}

pub fn hunt_players(
    starting_position: Position,
    map_size: MapSize,
    player_positions_entities: &[(Position, Entity)],
    hunter_entity: Entity,
    stone_wall_positions: &HashSet<Position>,
    impassable_positions: &HashSet<Position>,
    fire_positions: &HashSet<Position>,
    bomb_positions: &HashSet<Position>,
    assumed_bomb_range: usize,
    fireproof_positions: &HashSet<Position>,
    wall_of_death: Option<&WallOfDeath>,
) -> HashSet<Direction> {
    let mut rng = rand::thread_rng();
    let mut minf = (map_size.rows + map_size.columns) as f32;
    let mut target = None;
    let mut result = HashSet::default();

    if player_positions_entities.len() > 1 {
        let enemy_positions = player_positions_entities
            .iter()
            .filter_map(|t| {
                if matches!(t, &(_, e) if e != hunter_entity) {
                    Some(t.0)
                } else {
                    None
                }
            })
            .collect::<Vec<Position>>();
        if players_in_range(starting_position, &enemy_positions, 3) {
            for enemy_position in enemy_positions {
                let dist = dist(starting_position, enemy_position);
                if dist <= minf || (dist == minf && rng.gen_bool(0.5)) {
                    minf = dist;
                    target = Some(enemy_position);
                }
            }
        } else {
            let mut player_positions_entities_iter = player_positions_entities.iter();
            while let Some((_, entity)) = player_positions_entities_iter.next() {
                if *entity == hunter_entity {
                    let target_enemy_position =
                        if let Some(t) = player_positions_entities_iter.next() {
                            t.0
                        } else {
                            player_positions_entities.first().unwrap().0
                        };
                    minf = dist(starting_position, target_enemy_position);
                    target = Some(target_enemy_position);
                    break;
                }
            }
        }

        if minf > 1.0 {
            let target = target.unwrap();
            if (starting_position.y == target.y || starting_position.x == target.x)
                && stone_wall_positions.contains(&Position {
                    y: starting_position.y + (target.y - starting_position.y).signum(),
                    x: starting_position.x + (target.x - starting_position.x).signum(),
                })
            {
                if starting_position.y == target.y {
                    for direction in [Direction::Down, Direction::Up] {
                        let position = starting_position.offset(direction, 1);
                        if !impassable_positions.contains(&position)
                            && safe(
                                position,
                                fire_positions,
                                bomb_positions,
                                assumed_bomb_range,
                                fireproof_positions,
                                wall_of_death,
                                map_size,
                            )
                        {
                            result.insert(direction);
                        }
                    }
                } else {
                    for direction in [Direction::Right, Direction::Left] {
                        let position = starting_position.offset(direction, 1);
                        if !impassable_positions.contains(&position)
                            && safe(
                                position,
                                fire_positions,
                                bomb_positions,
                                assumed_bomb_range,
                                fireproof_positions,
                                wall_of_death,
                                map_size,
                            )
                        {
                            result.insert(direction);
                        }
                    }
                }
            }

            for direction in Direction::LIST {
                let position = starting_position.offset(direction, 1);
                if result.len() != 2
                    && !impassable_positions.contains(&position)
                    && safe(
                        position,
                        fire_positions,
                        bomb_positions,
                        assumed_bomb_range,
                        fireproof_positions,
                        wall_of_death,
                        map_size,
                    )
                    && dist(position, target) < minf
                {
                    result.insert(direction);
                }
            }
        }
    }

    result
}
