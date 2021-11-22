use std::{collections::HashSet, time::Duration};

use bevy::prelude::*;
use bevy_kira_audio::Audio;
use rand::Rng;

use crate::{
    common::{
        constants::{COLORS, PIXEL_SCALE},
        resources::{BaseColorMaterials, Fonts, GameOption, GameOptionStore},
    },
    game::{
        components::{
            BaseMaterial, Bomb, Fuse, HumanControlled, ImmortalBundle, ImmortalMaterial, Penguin,
            Perishable, Player, Position, UIComponent, UIRoot, Wall,
        },
        constants::{TILE_HEIGHT, TILE_WIDTH},
        events::ExplosionEvent,
        resources::{GameContext, HUDMaterials, MapSize, Textures, WorldID},
        types::{Cooldown, Direction},
        utils::{get_x, get_y, init_hud, spawn_map},
    },
    map_transition::MapTransitionInput,
    secret_mode::resources::{SecretModeContext, SecretModeDispatcherState, SecretModeInGameState},
    AppState,
};

use super::resources::SecretModeMusic;

pub fn setup_secret_mode(
    mut commands: Commands,
    audio: Res<Audio>,
    sounds: Res<SecretModeMusic>,
    mut textures: ResMut<Textures>,
    base_color_materials: Res<BaseColorMaterials>,
    hud_materials: Res<HUDMaterials>,
    fonts: Res<Fonts>,
) {
    // TODO: Audio will start playing only when the asset is loaded and decoded, which might be after
    // the mode is finished. However, waiting for it to load is VERY slow in debug builds, so there needs
    // to be a more granular loading wait implemented before the states that need certain assets.
    audio.play_looped(sounds.what_is_f.clone());

    const PATTERN: &str = r#"
*              *                  *****       ********************************************
 *             *                 *     *       *                     *       *            
  *            *        ***     *       *       *            **               *           
   *           *          *         *          *    *                 *                   
         *     ***        *     *       *     *    *     **      **                       
        *                 *      *     *          *              *         *              
       *                  *       *****            ************************************   
"#;

    let map_size = MapSize {
        rows: PATTERN.split('\n').count(),
        columns: 15,
    };

    let world_id = WorldID(rand::thread_rng().gen_range(1..=3));
    textures.set_map_textures(world_id);

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
            let hud_width = (map_size.columns * TILE_WIDTH) as f32;
            init_hud(
                parent,
                &hud_materials,
                &fonts,
                hud_width,
                world_id,
                false,
                false,
                Some(&|parent: &mut ChildBuilder| {
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(
                                    Val::Px(43.0 * PIXEL_SCALE as f32),
                                    Val::Px(2.0 * PIXEL_SCALE as f32),
                                ),
                                position_type: PositionType::Absolute,
                                position: Rect {
                                    left: Val::Px(hud_width / 2.0 - 20.0 * PIXEL_SCALE as f32),
                                    top: Val::Px(6.0 * PIXEL_SCALE as f32),
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
                                        "Hope you had fun with this little game! ^_^",
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
                                .insert(UIComponent);
                        });

                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(
                                    Val::Px(8.0 * PIXEL_SCALE as f32),
                                    Val::Px(2.0 * PIXEL_SCALE as f32),
                                ),
                                position_type: PositionType::Absolute,
                                position: Rect {
                                    left: Val::Px(hud_width / 2.0 + 10.0 * PIXEL_SCALE as f32),
                                    top: Val::Px(10.0 * PIXEL_SCALE as f32),
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
                                        "Now RUN!",
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
                                .insert(UIComponent);
                        });
                }),
            );
        });

    commands.insert_resource(world_id);
    commands.insert_resource(map_size);

    commands.insert_resource(SecretModeContext {
        dispatcher_state: SecretModeDispatcherState::Setup,
        in_game_state: SecretModeInGameState::Initial(Timer::from_seconds(2.5, false)),
        pattern: PATTERN,
    });
    commands.insert_resource(GameContext {
        pausable: false,
        // irrelevant in this mode
        reduced_loot: false,
    });
}

pub fn secret_mode_dispatch(
    mut commands: Commands,
    textures: Res<Textures>,
    mut secret_mode_context: ResMut<SecretModeContext>,
    map_size: Res<MapSize>,
    game_option_store: Res<GameOptionStore>,
    mut state: ResMut<State<AppState>>,
) {
    match secret_mode_context.dispatcher_state {
        SecretModeDispatcherState::Setup => {
            // map generation //

            // spawn player
            let player_spawn_position = Position {
                y: map_size.rows as isize / 2,
                x: 2,
            };
            let base_material = textures.get_penguin_texture(Penguin(0)).clone();
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
                .insert(Player)
                .insert(HumanControlled(0))
                .insert(player_spawn_position);

            let wall_entity_reveal_groups = spawn_map(
                &mut commands,
                &textures,
                *map_size,
                0.0,
                false,
                &[player_spawn_position],
                &[],
                false,
            );

            if game_option_store.get(GameOption::Transition) {
                secret_mode_context.dispatcher_state = SecretModeDispatcherState::MapTransition;
                commands.insert_resource(MapTransitionInput {
                    wall_entity_reveal_groups,
                });
                state.push(AppState::MapTransition).unwrap();
            } else {
                secret_mode_context.dispatcher_state = SecretModeDispatcherState::InGame;
                state.push(AppState::SecretModeInGame).unwrap();
            }
        }
        SecretModeDispatcherState::MapTransition => {
            secret_mode_context.dispatcher_state = SecretModeDispatcherState::InGame;
            state.push(AppState::SecretModeInGame).unwrap();
        }
        SecretModeDispatcherState::InGame => {
            commands.remove_resource::<SecretModeContext>();
            state.replace(AppState::MainMenu).unwrap();
        }
    }
}

pub fn update_secret_mode(
    mut commands: Commands,
    time: Res<Time>,
    textures: Res<Textures>,
    fonts: Res<Fonts>,
    map_size: Res<MapSize>,
    world_id: Res<WorldID>,
    mut secret_mode_context: ResMut<SecretModeContext>,
    mut state: ResMut<State<AppState>>,
    mut q: QuerySet<(
        QueryState<(Entity, &mut Position, &mut Transform), With<Bomb>>,
        QueryState<&Position, With<Wall>>,
    )>,
    mut query: Query<(Entity, &mut Handle<ColorMaterial>, &mut BaseMaterial), With<Player>>,
) {
    let pattern = secret_mode_context.pattern;

    loop {
        let new_state = match &mut secret_mode_context.in_game_state {
            SecretModeInGameState::Initial(timer) => {
                timer.tick(time.delta());
                if timer.finished() {
                    Some(SecretModeInGameState::Started {
                        move_cooldown: Cooldown::from_seconds(0.25),
                        round_progress: 0,
                        round: 0,
                    })
                } else {
                    None
                }
            }
            SecretModeInGameState::Started {
                move_cooldown,
                round_progress,
                round,
            } => {
                move_cooldown.tick(time.delta());
                if move_cooldown.ready() {
                    move_cooldown.trigger();

                    let walls: HashSet<Position> = q.q1().iter().copied().collect();
                    for (entity, mut position, mut transform) in q.q0().iter_mut() {
                        let new_position = position.offset(Direction::Left, 1);

                        if walls.contains(&new_position) {
                            commands.entity(entity).despawn_recursive();
                        } else {
                            *position = new_position;

                            let translation = &mut transform.translation;
                            translation.x = get_x(position.x);
                            translation.y = get_y(position.y);
                        }
                    }

                    let b = pattern
                        .split('\n')
                        .skip(1)
                        .take(7)
                        .map(|s| s.chars().nth(*round_progress as usize).unwrap() == '*');

                    for (i, b) in b.enumerate() {
                        if b {
                            let position = Position {
                                y: i as isize + 1,
                                x: map_size.columns as isize - 2,
                            };
                            commands
                                .spawn_bundle(SpriteBundle {
                                    material: textures.bomb.clone(),
                                    transform: Transform::from_xyz(
                                        get_x(position.x),
                                        get_y(position.y),
                                        25.0,
                                    ),
                                    sprite: Sprite::new(Vec2::new(
                                        TILE_WIDTH as f32,
                                        TILE_HEIGHT as f32,
                                    )),
                                    ..Default::default()
                                })
                                .insert(Bomb {
                                    owner: None,
                                    range: 3,
                                })
                                .insert(Perishable {
                                    timer: Timer::from_seconds(9999.0, false),
                                })
                                .insert(position)
                                .with_children(|parent| {
                                    let fuse_color =
                                        COLORS[if world_id.0 == 2 { 12 } else { 14 }].into();

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

                    *round_progress += 1;
                    if *round_progress >= pattern.split('\n').skip(1).take(7).next().unwrap().len()
                    {
                        *round += 1;
                        *round_progress = 0;

                        let new_material = textures.get_penguin_texture(Penguin(*round));
                        let (entity, mut color, mut base_material) = query.single_mut().unwrap();
                        *color = new_material.clone();
                        *base_material = BaseMaterial(new_material.clone());

                        commands
                            .entity(entity)
                            .insert_bundle(ImmortalBundle::default());

                        let current_duration = move_cooldown.duration();
                        if let Some(duration) =
                            current_duration.checked_sub(Duration::from_millis(30))
                        {
                            *move_cooldown = Cooldown::from_seconds(duration.as_secs_f32());
                        }
                    }
                }

                None
            }
            SecretModeInGameState::Stopping(timer) => {
                timer.tick(time.delta());

                if timer.just_finished() {
                    state.pop().unwrap();
                }

                None
            }
        };

        if let Some(new_state) = new_state {
            secret_mode_context.in_game_state = new_state;
        } else {
            break;
        }
    }
}

pub fn finish_secret_mode(
    mut commands: Commands,
    mut secret_mode_context: ResMut<SecretModeContext>,
    query: Query<(Entity, &Position), With<Player>>,
    query2: Query<(Entity, &Bomb, &Position)>,
    mut ev_explosion: EventWriter<ExplosionEvent>,
) {
    let (player_entity, player_position) = query.single().unwrap();
    if query2.iter().any(|(_, _, p)| *p == *player_position) {
        secret_mode_context.in_game_state =
            SecretModeInGameState::Stopping(Timer::from_seconds(0.5, false));

        commands.entity(player_entity).remove::<HumanControlled>();
        for (entity, bomb, position) in query2.iter() {
            commands.entity(entity).despawn_recursive();
            ev_explosion.send(ExplosionEvent(*position, bomb.range));
        }
    }
}

pub fn teardown(mut commands: Commands, audio: Res<Audio>, query: Query<Entity>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    commands.remove_resource::<WorldID>();
    commands.remove_resource::<MapSize>();
    commands.remove_resource::<SecretModeContext>();

    audio.stop();
}
