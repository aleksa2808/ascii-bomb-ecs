use bevy::prelude::*;
use rand::Rng;

use crate::game::{
    components::*,
    constants::{TILE_HEIGHT, TILE_WIDTH},
    resources::*,
    types::Cooldown,
    utils::{get_x, get_y},
};

use super::resources::Level;

pub fn format_hud_lives(lives: usize) -> String {
    format!("Lives:{}", lives)
}

pub fn format_hud_points(points: usize) -> String {
    format!("Points:{}", points)
}

pub fn spawn_story_mode_mobs(
    commands: &mut Commands,
    textures: &Textures,
    level: Level,
    world_id: WorldID,
    map_size: MapSize,
) -> Vec<Position> {
    // spawn mobs
    let mob_number = if let Level::Regular(num) = level {
        num + 1
    } else {
        1
    } + world_id.0;

    // TODO: currently hardcoded for 11x15
    let x = [
        map_size.columns - 4,
        map_size.columns - 2,
        11,
        5,
        1,
        map_size.columns - 6,
        map_size.columns - 6,
        7,
    ];
    let y = [
        map_size.rows - 8,
        1,
        map_size.rows - 2,
        map_size.rows - 6,
        9,
        5,
        map_size.rows - 4,
        7,
    ];
    let mut rng = rand::thread_rng();
    let bias = rng.gen::<usize>() % 20;

    let mut mob_spawn_positions = vec![];
    for i in 0..mob_number {
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

        let mob_spawn_position = Position {
            x: x[(i as usize + bias) % 6] as isize,
            y: y[(i as usize + bias) % 6] as isize,
        };
        mob_spawn_positions.push(mob_spawn_position);

        let mut ec = commands.spawn_bundle(SpriteBundle {
            material: base_material.clone(),
            transform: Transform::from_xyz(
                get_x(mob_spawn_position.x),
                get_y(mob_spawn_position.y),
                50.0,
            ),
            sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
            ..Default::default()
        });
        ec.insert(BaseMaterial(base_material))
            .insert(ImmortalMaterial(immortal_material))
            .insert(Player)
            .insert(MobAI::default())
            .insert(MoveCooldown(Cooldown::from_seconds(0.4)))
            .insert(Health {
                lives: 1,
                max_health: health,
                health,
            })
            .insert(mob_spawn_position)
            .insert(MeleeAttacker)
            .insert(TeamID(1))
            .insert(PointValue(point_value));

        if wall_hack {
            ec.insert(WallHack);
        }
    }

    mob_spawn_positions
}

pub fn spawn_story_mode_boss(
    commands: &mut Commands,
    textures: &Textures,
    world_id: WorldID,
    map_size: MapSize,
) -> (Position, Penguin) {
    let boss_spawn_position = Position {
        y: 3,
        x: map_size.columns as isize / 2,
    };
    let boss_penguin_tag = Penguin(3 + world_id.0);
    let base_material = textures.get_penguin_texture(boss_penguin_tag).clone();
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
        .insert(Player)
        .insert(boss_penguin_tag)
        .insert(BotAI)
        .insert(MoveCooldown(Cooldown::from_seconds(0.3)))
        .insert(Health {
            lives: 1,
            max_health: 2,
            health: 2,
        })
        .insert(boss_spawn_position)
        .insert(BombSatchel {
            bombs_available: 1 + world_id.0,
            bomb_range: 1 + world_id.0,
        })
        .insert(TeamID(1))
        .insert(PointValue(200));

    (boss_spawn_position, boss_penguin_tag)
}