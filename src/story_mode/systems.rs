use bevy::{app::Events, prelude::*, render::camera::Camera};

use crate::{
    common::{
        constants::{COLORS, PIXEL_SCALE},
        resources::{BaseColorMaterials, Fonts, GameOption, GameOptionStore, PersistentHighScores},
    },
    game::{components::*, constants::*, events::*, resources::*, utils::*},
    map_transition::MapTransitionInput,
    AppState,
};

use super::{
    components::Protagonist, constants::STORY_MODE_LEVEL_DURATION_SECS, resources::*, utils::*,
};

pub fn setup_story_mode(
    mut commands: Commands,
    mut textures: ResMut<Textures>,
    base_color_materials: Res<BaseColorMaterials>,
    hud_materials: Res<HUDMaterials>,
    fonts: Res<Fonts>,
) {
    let map_size = MapSize {
        rows: 11,
        columns: 15,
    };
    let world_id = WorldID(1);
    let player_lives = 2;
    let player_points = 0;

    textures.set_map_textures(world_id);

    // map generation //

    // spawn player
    let player_spawn_position = Position { y: 1, x: 1 };
    let player_penguin_tag = Penguin(0);
    let base_material = textures.get_penguin_texture(player_penguin_tag).clone();
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
        .insert(Protagonist)
        .insert(HumanControlled(0))
        .insert(Health {
            lives: player_lives,
            max_health: 1,
            health: 1,
        })
        .insert(player_spawn_position)
        .insert(BombSatchel {
            bombs_available: 3,
            bomb_range: 2,
        })
        .insert(player_penguin_tag)
        .insert(TeamID(0));

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
            init_hud(
                parent,
                &hud_materials,
                &fonts,
                (map_size.columns * TILE_WIDTH) as f32,
                world_id,
                true,
                true,
                Some(&|parent: &mut ChildBuilder| {
                    // lives display
                    parent
                        .spawn_bundle(TextBundle {
                            text: Text::with_section(
                                format_hud_lives(player_lives),
                                TextStyle {
                                    font: fonts.mono.clone(),
                                    font_size: 2.0 * PIXEL_SCALE as f32,
                                    color: COLORS[0].into(),
                                },
                                TextAlignment::default(),
                            ),
                            style: Style {
                                position_type: PositionType::Absolute,
                                position: Rect {
                                    top: Val::Px(12.0 * PIXEL_SCALE as f32),
                                    left: Val::Px(6.0 * PIXEL_SCALE as f32),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(UIComponent)
                        .insert(BottomLeftDisplay1);

                    // points display
                    parent
                        .spawn_bundle(TextBundle {
                            text: Text::with_section(
                                format_hud_points(player_points),
                                TextStyle {
                                    font: fonts.mono.clone(),
                                    font_size: 2.0 * PIXEL_SCALE as f32,
                                    color: COLORS[0].into(),
                                },
                                TextAlignment::default(),
                            ),
                            style: Style {
                                position_type: PositionType::Absolute,
                                position: Rect {
                                    top: Val::Px(12.0 * PIXEL_SCALE as f32),
                                    left: Val::Px(16.0 * PIXEL_SCALE as f32),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(UIComponent)
                        .insert(BottomLeftDisplay2);
                }),
            );
        });

    commands.insert_resource(StoryModeContext {
        state: StoryModeState::LevelSetup,
        level: Level::Regular(1),
        level_outcome: None,
        game_completed: false,
    });
    commands.insert_resource(GameContext {
        pausable: true,
        reduced_loot: false,
    });
    commands.insert_resource(GameScore(player_points));
    commands.insert_resource(GameTimer(Timer::from_seconds(
        STORY_MODE_LEVEL_DURATION_SECS as f32,
        false,
    )));
    commands.insert_resource(world_id);
    commands.insert_resource(map_size);
}

pub fn story_mode_manager(
    mut commands: Commands,
    mut textures: ResMut<Textures>,
    hud_materials: Res<HUDMaterials>,
    mut story_mode_context: ResMut<StoryModeContext>,
    mut game_score: ResMut<GameScore>,
    mut game_timer: ResMut<GameTimer>,
    mut world_id: ResMut<WorldID>,
    map_size: Res<MapSize>,
    game_option_store: Res<GameOptionStore>,
    persistent_high_scores: Res<PersistentHighScores>,
    mut state: ResMut<State<AppState>>,
    mut q: QuerySet<(
        QueryState<
            (
                Entity,
                &mut Handle<ColorMaterial>,
                &BaseMaterial,
                &mut BombSatchel,
            ),
            (With<Player>, With<Protagonist>),
        >,
        // doesn't need to be in here, but putting it outside throws errors (possibly because of too many arguments)
        QueryState<
            (Entity, &Penguin, &mut Position, &mut Transform, &mut Sprite),
            (With<Player>, With<Protagonist>),
        >,
        QueryState<&mut Handle<ColorMaterial>, With<HUDRoot>>,
    )>,
    mut q2: QuerySet<(
        QueryState<&mut Text, With<BottomLeftDisplay2>>,
        QueryState<&mut Text, With<GameTimerDisplay>>,
    )>,
    query: Query<Entity, With<PenguinPortrait>>,
    query2: Query<Entity, (Without<Camera>, Without<UIComponent>, Without<Protagonist>)>,
    query3: Query<&Bomb>,
) {
    loop {
        match story_mode_context.state {
            StoryModeState::LevelSetup => {
                let mut tmp = q.q1();
                let (
                    player_entity,
                    player_penguin_tag,
                    mut player_position,
                    mut transform,
                    mut sprite,
                ) = tmp.single_mut().unwrap();

                // move player to spawn
                *player_position = match story_mode_context.level {
                    Level::Regular(_) => Position { y: 1, x: 1 },
                    Level::BossRoom => Position {
                        y: map_size.rows as isize - 4,
                        x: map_size.columns as isize / 2,
                    },
                };
                let mut penguin_spawn_positions = vec![*player_position];
                let mut penguin_tags = vec![*player_penguin_tag];

                let translation = &mut transform.translation;
                translation.x = get_x(player_position.x);
                translation.y = get_y(player_position.y);

                // face the player right
                sprite.flip_x = false;

                // make the player temporarily immortal
                commands
                    .entity(player_entity)
                    .insert_bundle(ImmortalBundle::default());

                let mob_spawn_positions = spawn_story_mode_mobs(
                    &mut commands,
                    &textures,
                    story_mode_context.level,
                    *world_id,
                    *map_size,
                );

                if let Level::BossRoom = story_mode_context.level {
                    let (boss_spawn_position, boss_penguin_tag) =
                        spawn_story_mode_boss(&mut commands, &textures, *world_id, *map_size);
                    penguin_spawn_positions.push(boss_spawn_position);
                    penguin_tags.push(boss_penguin_tag);

                    let player = *player_penguin_tag;
                    let boss = boss_penguin_tag;
                    commands.insert_resource(BossSpeechScript::new(match world_id.0 {
                                    1 => vec![
                                        (boss, "You will never reach the north alive!"),
                                        (player, "Penguins can talk in this game? Cooooool!"),
                                        (boss, "Prepare to die!"),
                                    ],
                                    2 => vec![
                                        (boss, "How did you manage to get here?"),
                                        (
                                            player,
                                            "Dunno, killed some guys, went through some doors...",
                                        ),
                                        (player, "That kind of stuff..."),
                                        (boss, "Ugh, those fools!"),
                                        (
                                            boss,
                                            "Well, your journey stops here, you will never find the cloud city!",
                                        ),
                                        (
                                            player,
                                            "A cloud city? Nice, was getting kind of bored with this theme...",
                                        ),
                                    ],
                                    3 => vec![
                                        (boss, "Ah, I have been expecting you Agent P!"),
                                        (player, "Who even wrote these stupid dialogues?"),
                                        (boss, "nenexexedadada!"),
                                        (player, "Let's just get this over with..."),
                                    ],
                                    _ => unreachable!(), // TODO: feels like world_id should be an enum
                                }));
                }

                let wall_entity_reveal_groups = spawn_map(
                    &mut commands,
                    &textures,
                    *map_size,
                    if let Level::BossRoom = story_mode_context.level {
                        0.0
                    } else {
                        50.0
                    },
                    true,
                    &penguin_spawn_positions,
                    &mob_spawn_positions,
                    matches!(story_mode_context.level, Level::Regular(_)),
                );

                game_timer.0.reset();
                // update HUD clock
                q2.q1().single_mut().unwrap().sections[0].value =
                    format_hud_time(game_timer.0.duration().as_secs_f32().ceil() as usize);

                story_mode_context.level_outcome = None;

                if game_option_store.get(GameOption::Transition) {
                    story_mode_context.state = StoryModeState::MapTransition;
                    commands.insert_resource(MapTransitionInput {
                        wall_entity_reveal_groups,
                    });
                    state.push(AppState::MapTransition).unwrap();
                } else if let Level::BossRoom = story_mode_context.level {
                    story_mode_context.state = StoryModeState::BossSpeech;
                    state.push(AppState::BossSpeech).unwrap();
                } else {
                    story_mode_context.state = StoryModeState::InGame;
                    state.push(AppState::StoryModeInGame).unwrap();
                }
            }
            StoryModeState::MapTransition => {
                if let Level::BossRoom = story_mode_context.level {
                    story_mode_context.state = StoryModeState::BossSpeech;
                    state.push(AppState::BossSpeech).unwrap();
                } else {
                    story_mode_context.state = StoryModeState::InGame;
                    state.push(AppState::StoryModeInGame).unwrap();
                }
            }
            StoryModeState::BossSpeech => {
                story_mode_context.state = StoryModeState::InGame;
                state.push(AppState::StoryModeInGame).unwrap();
            }
            StoryModeState::InGame => {
                match story_mode_context.level_outcome {
                    Some(LevelOutcome::Win) => {
                        if let Level::Regular(num) = story_mode_context.level {
                            println!("Level {}x{} completed!", world_id.0, num);
                        } else {
                            println!("World {} boss defeated!", world_id.0);
                        }

                        // add 5 points for each second left on the clock
                        game_score.0 += 5
                            * (game_timer.0.duration() - game_timer.0.elapsed()).as_secs() as usize;
                        // update HUD points
                        q2.q0().single_mut().unwrap().sections[0].value =
                            format_hud_points(game_score.0);

                        match (story_mode_context.level, world_id.0) {
                            (Level::BossRoom, 3) => {
                                game_score.0 += 2000;
                                story_mode_context.game_completed = true;
                                println!("Game completed! Final score: {}", game_score.0);

                                story_mode_context.state = StoryModeState::ScoreCheck;
                                continue;
                            }
                            (Level::BossRoom, _) => {
                                world_id.0 += 1;
                                story_mode_context.level = Level::Regular(1);
                                *q.q2().single_mut().unwrap() =
                                    hud_materials.get_background_material(*world_id).clone();
                                textures.set_map_textures(*world_id);
                            }
                            (Level::Regular(num), _) => {
                                if num < 4 {
                                    story_mode_context.level = Level::Regular(num + 1);
                                } else {
                                    story_mode_context.level = Level::BossRoom
                                }
                            }
                        };

                        let mut tmp = q.q0();
                        let (player_entity, mut player_material, base_material, mut bomb_satchel) =
                            tmp.single_mut().unwrap();

                        // reset the player's texture (clears immortality animation effects)
                        *player_material = base_material.0.clone();

                        let unexploded_player_bombs = query3
                            .iter()
                            .filter(|b| matches!(b.owner, Some(entity) if entity == player_entity))
                            .count();

                        for entity in query2.iter() {
                            commands.entity(entity).despawn_recursive();
                        }

                        // clear penguin portraits
                        for entity in query.iter() {
                            commands.entity(entity).despawn_recursive();
                        }

                        // bomb refill
                        bomb_satchel.bombs_available += unexploded_player_bombs;

                        story_mode_context.state = StoryModeState::LevelSetup;
                        continue;
                    }
                    Some(LevelOutcome::Loss) => {
                        println!("Game over! Final score: {}", game_score.0);
                        story_mode_context.state = StoryModeState::ScoreCheck;
                        continue;
                    }
                    None => {
                        story_mode_context.state = StoryModeState::ScoreCheck;
                        continue;
                    }
                }
            }
            StoryModeState::ScoreCheck => {
                story_mode_context.state = StoryModeState::HighScoreNameInput;
                if game_score.0 > persistent_high_scores.entry_threshold() {
                    state.push(AppState::HighScoreNameInput).unwrap();
                } else {
                    // skip to the step below where we choose the next state
                    continue;
                }
            }
            StoryModeState::HighScoreNameInput => {
                if story_mode_context.game_completed {
                    state.replace(AppState::SecretMode).unwrap();
                } else {
                    state.replace(AppState::MainMenu).unwrap();
                }
            }
        }
        break;
    }
}

pub fn hud_lives_indicator_update(
    mut query: Query<&mut Text, With<BottomLeftDisplay1>>,
    query2: Query<&Health, (With<Protagonist>, Changed<Health>)>,
) {
    // need to be careful around death edge cases here
    if let Ok(player) = query2.single() {
        query.single_mut().unwrap().sections[0].value = format_hud_lives(player.lives);
    }
}

pub fn hud_points_indicator_update(
    game_score: Res<GameScore>,
    mut query: Query<&mut Text, With<BottomLeftDisplay2>>,
) {
    if game_score.is_changed() {
        query.single_mut().unwrap().sections[0].value = format_hud_points(game_score.0);
    }
}

pub fn finish_level(
    mut story_mode_context: ResMut<StoryModeContext>,
    game_timer: Res<GameTimer>,
    mut q: QuerySet<(
        QueryState<(&mut Position, &TeamID), (With<Player>, With<Protagonist>)>,
        QueryState<&Position, With<Exit>>,
    )>,
    query: Query<&Protagonist>,
    query2: Query<&TeamID, With<Player>>,
    mut state: ResMut<State<AppState>>,
    mut keyboard_input: ResMut<Input<KeyCode>>,
) {
    let mut level_outcome = None;

    match story_mode_context.level {
        Level::Regular(_) => {
            // if an exit is spawned...
            if let Ok(exit_position) = q.q1().single().map(|p| *p) {
                // ...check if a protagonist reached it when all the enemies are dead
                if q.q0().iter_mut().any(|(pp, ptid)| {
                    *pp == exit_position && !query2.iter().any(|tid| tid.0 != ptid.0)
                }) {
                    level_outcome = Some(LevelOutcome::Win);
                }
            }
        }
        Level::BossRoom => {
            // if a protagonist killed all the enemies
            if q.q0()
                .iter_mut()
                .any(|(_, ptid)| !query2.iter().any(|tid| tid.0 != ptid.0))
            {
                level_outcome = Some(LevelOutcome::Win);
            }
        }
    }

    // if the timer expired or the protagonist is dead
    if game_timer.0.finished() || query.iter().count() == 0 {
        level_outcome = Some(LevelOutcome::Loss);
    }

    // TODO: used for debugging, remove
    if keyboard_input.just_pressed(KeyCode::F) {
        level_outcome = Some(LevelOutcome::Win);
        keyboard_input.reset(KeyCode::F);
    }

    if level_outcome.is_some() {
        story_mode_context.level_outcome = level_outcome;
        state.overwrite_pop().unwrap();
    }
}

pub fn setup_boss_speech(
    mut commands: Commands,
    hud_materials: Res<HUDMaterials>,
    textures: Res<Textures>,
    boss_speech_script: Res<BossSpeechScript>,
    fonts: Res<Fonts>,
    query: Query<Entity, With<HUDRoot>>,
) {
    let mut speech_box = None;
    let mut speaker_portrait = None;
    let mut speech_text = None;

    commands
        .entity(query.single().unwrap())
        .with_children(|parent| {
            speech_box = Some(
                parent
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                            position_type: PositionType::Absolute,
                            position: Rect {
                                left: Val::Px(0.0),
                                top: Val::Px(0.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        material: hud_materials.black.clone(),
                        ..Default::default()
                    })
                    .insert(UIComponent)
                    .with_children(|parent| {
                        // dialog border
                        parent
                            .spawn_bundle(TextBundle {
                                text: Text::with_section(
                                    r#"
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                        │
│                                                                                        │
│                                                                                        │
│                                                                                        │
│                                                                                        │
└────────────────────────────────────────────────────────────────────────────────────────┘
"#,
                                    TextStyle {
                                        font: fonts.mono.clone(),
                                        font_size: 2.0 * PIXEL_SCALE as f32,
                                        color: COLORS[15].into(), // TODO: is this the right color?
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

                        // player portrait
                        parent
                            .spawn_bundle(NodeBundle {
                                style: Style {
                                    size: Size::new(
                                        Val::Px(8.0 * PIXEL_SCALE as f32),
                                        Val::Px(10.0 * PIXEL_SCALE as f32),
                                    ),
                                    position_type: PositionType::Absolute,
                                    position: Rect {
                                        left: Val::Px(4.0 * PIXEL_SCALE as f32),
                                        top: Val::Px(2.0 * PIXEL_SCALE as f32),
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
                            .insert(UIComponent)
                            .with_children(|parent| {
                                parent
                                    .spawn_bundle(NodeBundle {
                                        style: Style {
                                            size: Size::new(
                                                Val::Percent(100.0),
                                                Val::Percent(100.0),
                                            ),
                                            ..Default::default()
                                        },
                                        material: hud_materials.portrait_background_color.clone(),
                                        ..Default::default()
                                    })
                                    .insert(UIComponent)
                                    .with_children(|parent| {
                                        speaker_portrait = Some(
                                            parent
                                                .spawn_bundle(ImageBundle {
                                                    style: Style {
                                                        size: Size::new(
                                                            Val::Percent(100.0),
                                                            Val::Percent(100.0),
                                                        ),
                                                        ..Default::default()
                                                    },
                                                    material: textures
                                                        .get_penguin_texture(
                                                            boss_speech_script
                                                                .get_current_speaker(),
                                                        )
                                                        .clone(),
                                                    ..Default::default()
                                                })
                                                .insert(UIComponent)
                                                .id(),
                                        );
                                    });
                            });

                        // speech text
                        speech_text = Some(
                            parent
                                .spawn_bundle(TextBundle {
                                    text: Text::with_section(
                                        boss_speech_script.get_current_line_state(),
                                        TextStyle {
                                            font: fonts.mono.clone(),
                                            font_size: 2.0 * PIXEL_SCALE as f32,
                                            color: COLORS[15].into(), // TODO: is this the right color?
                                        },
                                        TextAlignment::default(),
                                    ),
                                    style: Style {
                                        position_type: PositionType::Absolute,
                                        position: Rect {
                                            top: Val::Px(6.0 * PIXEL_SCALE as f32),
                                            left: Val::Px(16.0 * PIXEL_SCALE as f32),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })
                                .insert(UIComponent)
                                .id(),
                        );
                    })
                    .id(),
            );
        });

    commands.insert_resource(BossSpeechBoxEntities {
        speech_box: speech_box.unwrap(),
        speaker_portrait: speaker_portrait.unwrap(),
        speech_text: speech_text.unwrap(),
    });
}

pub fn boss_speech_update(
    mut commands: Commands,
    time: Res<Time>,
    textures: Res<Textures>,
    mut boss_speech_script: ResMut<BossSpeechScript>,
    boss_speech_box_entities: Res<BossSpeechBoxEntities>,
    mut keyboard_input: ResMut<Input<KeyCode>>,
    mut state: ResMut<State<AppState>>,
    mut query: Query<&mut Text>,
    mut query2: Query<&mut Handle<ColorMaterial>>,
) {
    boss_speech_script.tick(time.delta());

    if keyboard_input.just_pressed(KeyCode::Space) {
        if boss_speech_script.line_in_progress() {
            boss_speech_script.complete_current_line();
        } else if boss_speech_script.advance_script().is_ok() {
            *query2
                .get_mut(boss_speech_box_entities.speaker_portrait)
                .unwrap() = textures
                .get_penguin_texture(boss_speech_script.get_current_speaker())
                .clone();
        } else {
            commands
                .entity(boss_speech_box_entities.speech_box)
                .despawn_recursive();

            commands.remove_resource::<BossSpeechBoxEntities>();
            commands.remove_resource::<BossSpeechScript>();

            state.pop().unwrap();
            keyboard_input.reset(KeyCode::Space);
            return;
        }
    }

    query
        .get_mut(boss_speech_box_entities.speech_text)
        .unwrap()
        .sections[0]
        .value = boss_speech_script.get_current_line_state().to_string();
}

pub fn setup_high_score_name_input(
    mut commands: Commands,
    hud_materials: Res<HUDMaterials>,
    fonts: Res<Fonts>,
    query: Query<Entity, With<UIRoot>>,
    map_size: Res<MapSize>,
) {
    let mut input_box = None;
    let mut name_text = None;

    commands
        .entity(query.single().unwrap())
        .with_children(|parent| {
            input_box = Some(
                parent
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(
                                Val::Px(30.0 * PIXEL_SCALE as f32),
                                Val::Px(10.0 * PIXEL_SCALE as f32),
                            ),
                            position_type: PositionType::Absolute,
                            position: Rect {
                                left: Val::Px(
                                    ((map_size.columns * (TILE_WIDTH / PIXEL_SCALE) / 2 - 15)
                                        * PIXEL_SCALE) as f32,
                                ),
                                top: Val::Px(
                                    // messy equation that produces the same results as the C code (integer divisions)
                                    ((((HUD_HEIGHT + map_size.rows * TILE_HEIGHT) / PIXEL_SCALE)
                                        / 4
                                        * 2
                                        - 6)
                                        * PIXEL_SCALE) as f32,
                                ),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        material: hud_materials.black.clone(),
                        ..Default::default()
                    })
                    .insert(UIComponent)
                    .with_children(|parent| {
                        // dialog border
                        parent
                            .spawn_bundle(TextBundle {
                                text: Text::with_section(
                                    r#"
┌────────────────────────────┐
│                            │
│ Name:                      │
│                            │
└────────────────────────────┘
"#,
                                    TextStyle {
                                        font: fonts.mono.clone(),
                                        font_size: 2.0 * PIXEL_SCALE as f32,
                                        color: COLORS[15].into(), // TODO: is this the right color?
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

                        // name text
                        name_text = Some(
                            parent
                                .spawn_bundle(TextBundle {
                                    text: Text::with_section(
                                        "",
                                        TextStyle {
                                            font: fonts.mono.clone(),
                                            font_size: 2.0 * PIXEL_SCALE as f32,
                                            color: COLORS[15].into(), // TODO: is this the right color?
                                        },
                                        TextAlignment::default(),
                                    ),
                                    style: Style {
                                        position_type: PositionType::Absolute,
                                        position: Rect {
                                            top: Val::Px(4.0 * PIXEL_SCALE as f32),
                                            left: Val::Px(8.0 * PIXEL_SCALE as f32),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })
                                .insert(UIComponent)
                                .id(),
                        );
                    })
                    .id(),
            );
        });

    commands.insert_resource(HighScoreNameInputContext {
        input_box: input_box.unwrap(),
        name_text: name_text.unwrap(),
    });
}

pub fn high_score_name_input_update(
    mut commands: Commands,
    context: Res<HighScoreNameInputContext>,
    mut keyboard_input: ResMut<Input<KeyCode>>,
    mut char_input_events: EventReader<ReceivedCharacter>,
    mut persistent_high_scores: ResMut<PersistentHighScores>,
    game_score: Res<GameScore>,
    mut query: Query<&mut Text>,
    mut state: ResMut<State<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        persistent_high_scores.insert_score(String::from("<unnamed_player>"), game_score.0);
        commands.remove_resource::<HighScoreNameInputContext>();
        state.pop().unwrap();
        keyboard_input.reset(KeyCode::Escape);
        return;
    }

    let name = &mut query.get_mut(context.name_text).unwrap().sections[0].value;

    for event in char_input_events.iter() {
        if name.len() < 20 && (event.char == ' ' || event.char.is_ascii_graphic()) {
            name.push(if event.char == ' ' { '_' } else { event.char });
        }
    }

    if keyboard_input.just_pressed(KeyCode::Back) {
        name.pop();
    }

    if keyboard_input.just_pressed(KeyCode::Return) {
        let name = if name.is_empty() {
            String::from("<unnamed_player>")
        } else {
            name.clone()
        };

        persistent_high_scores.insert_score(name, game_score.0);
        commands.remove_resource::<HighScoreNameInputContext>();
        state.pop().unwrap();
        keyboard_input.reset(KeyCode::Return);
    }
}

pub fn teardown(
    mut commands: Commands,
    query: Query<Entity>,
    mut player_action_events: ResMut<Events<PlayerActionEvent>>,
    mut explosion_events: ResMut<Events<ExplosionEvent>>,
    mut burn_events: ResMut<Events<BurnEvent>>,
    mut damage_events: ResMut<Events<DamageEvent>>,
) {
    // clear entities
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
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

    // story mode
    commands.remove_resource::<StoryModeContext>();
    commands.remove_resource::<GameScore>();
    commands.remove_resource::<ExitPosition>();
}
