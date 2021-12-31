use std::collections::HashSet;

use bevy::prelude::*;
use rand::{
    prelude::{IteratorRandom, SliceRandom},
    Rng,
};

use crate::common::{
    constants::{COLORS, PIXEL_SCALE},
    resources::Fonts,
};

use super::{
    components::*,
    constants::*,
    resources::*,
    types::{Direction, *},
};

pub fn get_x(x: isize) -> f32 {
    TILE_WIDTH as f32 / 2.0 + (x * TILE_WIDTH as isize) as f32
}

pub fn get_y(y: isize) -> f32 {
    -(TILE_HEIGHT as f32 / 2.0 + (HUD_HEIGHT as isize + y * TILE_HEIGHT as isize) as f32)
}

pub fn format_hud_time(remaining_seconds: usize) -> String {
    format!(
        "{:02}:{:02}",
        remaining_seconds / 60,
        remaining_seconds % 60
    )
}

pub fn init_hud(
    parent: &mut ChildBuilder,
    hud_materials: &HUDMaterials,
    fonts: &Fonts,
    width: f32,
    world_id: WorldID,
    with_penguin_portrait_display: bool,
    with_clock: bool,
    extra_item_fn: Option<&dyn Fn(&mut ChildBuilder)>,
) {
    let mut ec = parent.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Px(width), Val::Px(HUD_HEIGHT as f32)),
            position_type: PositionType::Absolute,
            position: Rect {
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..Default::default()
            },
            ..Default::default()
        },
        material: hud_materials.get_background_material(world_id).clone(),
        ..Default::default()
    });

    ec.insert(UIComponent)
        .insert(HUDRoot)
        .with_children(|parent| {
            if let Some(extra_item_fn) = extra_item_fn {
                extra_item_fn(parent);
            }

            if with_clock {
                // clock / pause indicator
                parent
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(
                                Val::Px(5.0 * PIXEL_SCALE as f32),
                                Val::Px(2.0 * PIXEL_SCALE as f32),
                            ),
                            position_type: PositionType::Absolute,
                            position: Rect {
                                left: Val::Px(width / 2.0 - 3.0 * PIXEL_SCALE as f32),
                                top: Val::Px(12.0 * PIXEL_SCALE as f32),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        material: hud_materials.black.clone(),
                        ..Default::default()
                    })
                    .insert(UIComponent)
                    .with_children(|parent| {
                        parent
                            .spawn_bundle(TextBundle {
                                text: Text::with_section(
                                    "",
                                    TextStyle {
                                        font: fonts.mono.clone(),
                                        font_size: 2.0 * PIXEL_SCALE as f32,
                                        color: COLORS[15].into(),
                                    },
                                    TextAlignment::default(),
                                ),
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    position: Rect {
                                        top: Val::Px(0.0),
                                        left: Val::Px(0.0),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .insert(UIComponent)
                            .insert(GameTimerDisplay);
                    });
            }
        });

    if with_penguin_portrait_display {
        ec.insert(PenguinPortraitDisplay);
    }
}

pub fn init_penguin_portraits(
    parent: &mut ChildBuilder,
    penguin_tags: &[Penguin],
    hud_materials: &HUDMaterials,
    game_materials: &GameMaterials,
) {
    for penguin in penguin_tags {
        parent
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(
                        Val::Px(8.0 * PIXEL_SCALE as f32),
                        Val::Px(10.0 * PIXEL_SCALE as f32),
                    ),
                    position_type: PositionType::Absolute,
                    position: Rect {
                        left: Val::Px(((5 + 12 * penguin.0) * PIXEL_SCALE) as f32),
                        top: Val::Px(PIXEL_SCALE as f32),
                        ..Default::default()
                    },
                    border: Rect {
                        left: Val::Px(PIXEL_SCALE as f32),
                        top: Val::Px(PIXEL_SCALE as f32),
                        right: Val::Px(PIXEL_SCALE as f32),
                        bottom: Val::Px(PIXEL_SCALE as f32),
                    },
                    ..Default::default()
                },
                material: hud_materials.portrait_border_color.clone(),
                ..Default::default()
            })
            .insert(PenguinPortrait(*penguin))
            .insert(UIComponent)
            .with_children(|parent| {
                parent
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                            ..Default::default()
                        },
                        material: hud_materials.portrait_background_color.clone(),
                        ..Default::default()
                    })
                    .insert(UIComponent)
                    .with_children(|parent| {
                        parent
                            .spawn_bundle(ImageBundle {
                                style: Style {
                                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                    ..Default::default()
                                },
                                material: game_materials.get_penguin_material(*penguin).clone(),
                                ..Default::default()
                            })
                            .insert(UIComponent);
                    });
            });
    }
}

pub fn spawn_map(
    commands: &mut Commands,
    game_materials: &GameMaterials,
    map_size: MapSize,
    percent_of_passable_positions_to_fill: f32,
    spawn_middle_blocks: bool,
    penguin_spawn_positions: &[Position],
    mob_spawn_positions: &[Position],
    spawn_exit: bool,
) -> Vec<Vec<Entity>> {
    let mut rng = rand::thread_rng();

    // place empty/passable tiles
    for j in 0..map_size.rows {
        for i in 0..map_size.columns {
            commands.spawn_bundle(SpriteBundle {
                material: game_materials.get_map_materials().empty.clone(),
                transform: Transform::from_xyz(get_x(i as isize), get_y(j as isize), 0.0),
                sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                ..Default::default()
            });
        }
    }

    // spawn walls
    let mut stone_wall_spawn_groups = vec![];
    for i in 0..map_size.rows {
        let left = Position {
            y: i as isize,
            x: 0,
        };
        let right = Position {
            y: i as isize,
            x: (map_size.columns - 1) as isize,
        };
        stone_wall_spawn_groups.push(vec![left, right]);
    }
    for i in 1..map_size.columns - 1 {
        let top = Position {
            y: 0,
            x: i as isize,
        };
        let bottom = Position {
            y: (map_size.rows - 1) as isize,
            x: i as isize,
        };
        stone_wall_spawn_groups.push(vec![top, bottom]);
    }
    // checkered middle
    if spawn_middle_blocks {
        for i in (2..map_size.rows).step_by(2) {
            for j in (2..map_size.columns).step_by(2) {
                let position = Position {
                    y: i as isize,
                    x: j as isize,
                };
                stone_wall_spawn_groups.push(vec![position]);
            }
        }
    }

    let mut wall_entity_reveal_groups = vec![];
    for spawn_group in stone_wall_spawn_groups.iter() {
        let mut reveal_group = vec![];
        for position in spawn_group {
            let entity = commands
                .spawn_bundle(SpriteBundle {
                    material: game_materials.get_map_materials().wall.clone(),
                    transform: Transform::from_xyz(get_x(position.x), get_y(position.y), 0.0),
                    sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                    ..Default::default()
                })
                .insert(Wall)
                .insert(Solid)
                .insert(*position)
                .id();
            reveal_group.push(entity);
        }
        wall_entity_reveal_groups.push(reveal_group);
    }

    let stone_wall_positions: HashSet<Position> =
        stone_wall_spawn_groups.iter().flatten().copied().collect();
    let mut destructible_wall_potential_positions: HashSet<Position> = (0..map_size.rows)
        .flat_map(|y| {
            (0..map_size.columns).map(move |x| Position {
                y: y as isize,
                x: x as isize,
            })
        })
        .filter(|p| !stone_wall_positions.contains(p))
        .collect();

    let number_of_passable_positions = destructible_wall_potential_positions.len();

    // reserve room for the penguins (cross-shaped)
    for penguin_spawn_position in penguin_spawn_positions {
        destructible_wall_potential_positions.remove(penguin_spawn_position);
        for position in Direction::LIST
            .iter()
            .map(|direction| penguin_spawn_position.offset(*direction, 1))
        {
            destructible_wall_potential_positions.remove(&position);
        }
    }

    // reserve room for the mobs (line-shaped)
    for mob_spawn_position in mob_spawn_positions {
        destructible_wall_potential_positions.remove(mob_spawn_position);

        for direction in [
            [Direction::Left, Direction::Right],
            [Direction::Up, Direction::Down],
        ]
        .choose(&mut rng)
        .unwrap()
        {
            for j in 1..3 {
                let position = mob_spawn_position.offset(*direction, j);
                if stone_wall_positions.contains(&position) {
                    break;
                }
                destructible_wall_potential_positions.remove(&position);
            }
        }
    }

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
        let entity = commands
            .spawn_bundle(SpriteBundle {
                material: game_materials.get_map_materials().destructible_wall.clone(),
                transform: Transform::from_xyz(get_x(position.x), get_y(position.y), 0.0),
                sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                ..Default::default()
            })
            .insert(Wall)
            .insert(Solid)
            .insert(Destructible)
            .insert(*position)
            .id();
        wall_entity_reveal_groups.push(vec![entity]);
    }

    if spawn_exit {
        commands.insert_resource(ExitPosition(
            *destructible_wall_positions.choose(&mut rng).unwrap(),
        ));
    }

    wall_entity_reveal_groups
}

pub fn generate_item_at_position(
    position: Position,
    commands: &mut Commands,
    game_materials: &GameMaterials,
    reduced_loot: bool,
) {
    let r = rand::thread_rng().gen::<usize>() % 100;

    /* "Loot tables" */
    let item = if !reduced_loot {
        match r {
            _ if r < 50 => Item::Upgrade(Upgrade::BombsUp),
            50..=79 => Item::Upgrade(Upgrade::RangeUp),
            80..=89 => Item::Power(Power::BombPush),
            90..=93 => Item::Upgrade(Upgrade::LivesUp),
            94..=97 => Item::Power(Power::WallHack),
            _ if r >= 98 => Item::Power(Power::Immortal),
            _ => unreachable!(),
        }
    } else {
        match r {
            _ if r < 50 => Item::Upgrade(Upgrade::BombsUp),
            50..=89 => Item::Upgrade(Upgrade::RangeUp),
            _ if r >= 90 => Item::Power(Power::BombPush),
            _ => unreachable!(),
        }
    };

    commands
        .spawn_bundle(SpriteBundle {
            material: match item {
                Item::Upgrade(Upgrade::BombsUp) => game_materials.bombs_up.clone(),
                Item::Upgrade(Upgrade::RangeUp) => game_materials.range_up.clone(),
                Item::Upgrade(Upgrade::LivesUp) => game_materials.lives_up.clone(),
                Item::Power(Power::WallHack) => game_materials.wall_hack.clone(),
                Item::Power(Power::BombPush) => game_materials.bomb_push.clone(),
                Item::Power(Power::Immortal) => game_materials.immortal.clone(),
            },
            transform: Transform::from_xyz(get_x(position.x), get_y(position.y), 20.0),
            sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
            ..Default::default()
        })
        .insert(position)
        .insert(item);
}
