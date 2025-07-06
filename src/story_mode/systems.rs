use bevy::{ecs::event::Events, prelude::*, render::camera::Camera};

use crate::{
    common::{
        constants::{COLORS, PIXEL_SCALE},
        resources::{
            Fonts, GameOption, GameOptionStore, InputActionStatusTracker, PersistentHighScores,
        },
        types::InputAction,
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
    mut game_textures: ResMut<GameTextures>,
    hud_colors: Res<HUDColors>,
    fonts: Res<Fonts>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let map_size = MapSize {
        rows: 11,
        columns: 15,
    };
    let world_id = WorldID(1);
    let player_lives = 5;
    let player_points = 0;

    game_textures.set_map_textures(world_id);

    // map generation //

    // spawn player
    let player_spawn_position = Position { y: 1, x: 1 };
    let player_penguin_tag = Penguin(0);
    let base_texture = game_textures
        .get_penguin_texture(player_penguin_tag)
        .clone();
    let immortal_texture = game_textures.immortal_penguin.clone();
    commands.spawn((
        SpriteBundle {
            texture: base_texture.clone(),
            transform: Transform::from_xyz(
                get_x(player_spawn_position.x),
                get_y(player_spawn_position.y),
                50.0,
            ),
            sprite: Sprite {
                custom_size: Some(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                ..Default::default()
            },
            ..Default::default()
        },
        BaseTexture(base_texture),
        ImmortalTexture(immortal_texture),
        Player,
        Protagonist,
        HumanControlled(0),
        Health {
            lives: player_lives,
            max_health: 1,
            health: 1,
        },
        player_spawn_position,
        SpawnPosition(player_spawn_position),
        BombSatchel {
            bombs_available: 1,
            bomb_range: 1,
        },
        player_penguin_tag,
        TeamID(0),
    ));

    // spawn HUD
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..Default::default()
                },
                background_color: Color::NONE.into(),
                ..Default::default()
            },
            UIRoot,
            UIComponent,
        ))
        .with_children(|parent| {
            init_hud(
                parent,
                &hud_colors,
                &fonts,
                (map_size.columns * TILE_WIDTH) as f32,
                world_id,
                true,
                true,
                Some(&|parent: &mut ChildBuilder| {
                    // lives display
                    parent.spawn((
                        TextBundle {
                            text: Text::from_section(
                                format_hud_lives(player_lives),
                                TextStyle {
                                    font: fonts.mono.clone(),
                                    font_size: 2.0 * PIXEL_SCALE as f32,
                                    color: COLORS[0].into(),
                                },
                            ),
                            style: Style {
                                position_type: PositionType::Absolute,
                                top: Val::Px(12.0 * PIXEL_SCALE as f32),
                                left: Val::Px(6.0 * PIXEL_SCALE as f32),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        UIComponent,
                        BottomLeftDisplay1,
                    ));

                    // points display
                    parent.spawn((
                        TextBundle {
                            text: Text::from_section(
                                format_hud_points(player_points),
                                TextStyle {
                                    font: fonts.mono.clone(),
                                    font_size: 2.0 * PIXEL_SCALE as f32,
                                    color: COLORS[0].into(),
                                },
                            ),
                            style: Style {
                                position_type: PositionType::Absolute,
                                top: Val::Px(12.0 * PIXEL_SCALE as f32),
                                left: Val::Px(16.0 * PIXEL_SCALE as f32),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        UIComponent,
                        BottomLeftDisplay2,
                    ));
                }),
            );
        });

    commands.insert_resource(StoryModeContext {
        state: StoryModeState::LevelSetup,
        level: Level::Regular(1),
        level_outcome: None,
        game_completed: false,
    });

    let game_mode_manager_state = AppState::StoryModeManager;
    commands.insert_resource(GameContext {
        pausable: true,
        reduced_loot: false,
        game_mode_manager_state,
    });
    commands.insert_resource(GameScore(player_points));
    commands.insert_resource(GameTimer(Timer::from_seconds(
        STORY_MODE_LEVEL_DURATION_SECS as f32,
        TimerMode::Once,
    )));
    commands.insert_resource(world_id);
    commands.insert_resource(map_size);

    next_state.set(game_mode_manager_state);
}

pub fn story_mode_manager(
    mut commands: Commands,
    mut game_textures: ResMut<GameTextures>,
    hud_colors: Res<HUDColors>,
    mut story_mode_context: ResMut<StoryModeContext>,
    mut game_score: ResMut<GameScore>,
    mut game_timer: ResMut<GameTimer>,
    mut world_id: ResMut<WorldID>,
    map_size: Res<MapSize>,
    game_option_store: Res<GameOptionStore>,
    persistent_high_scores: Res<PersistentHighScores>,
    mut next_state: ResMut<NextState<AppState>>,
    mut p: ParamSet<(
        Query<
            (Entity, &mut Handle<Image>, &BaseTexture, &mut BombSatchel),
            (With<Player>, With<Protagonist>),
        >,
        // doesn't need to be in here, but putting it outside throws errors (possibly because of too many arguments)
        Query<
            (
                Entity,
                &Penguin,
                &mut Position,
                &mut SpawnPosition,
                &mut Transform,
                &mut Sprite,
            ),
            (With<Player>, With<Protagonist>),
        >,
        Query<&mut BackgroundColor, With<HUDRoot>>,
    )>,
    mut p2: ParamSet<(
        Query<&mut Text, With<BottomLeftDisplay2>>,
        Query<&mut Text, With<GameTimerDisplay>>,
    )>,
    query: Query<Entity, With<PenguinPortrait>>,
    query2: Query<
        Entity,
        (
            Without<Window>,
            Without<Camera>,
            Without<UIComponent>,
            Without<Protagonist>,
        ),
    >,
    query3: Query<&Bomb>,
) {
    loop {
        match story_mode_context.state {
            StoryModeState::LevelSetup => {
                let mut tmp = p.p1();
                let (
                    player_entity,
                    player_penguin_tag,
                    mut player_position,
                    mut player_spawn_position,
                    mut transform,
                    mut sprite,
                ) = tmp.single_mut();

                // move player to spawn
                *player_position = match story_mode_context.level {
                    Level::Regular(_) => Position { y: 1, x: 1 },
                    Level::BossRoom => Position {
                        y: map_size.rows as isize - 4,
                        x: map_size.columns as isize / 2,
                    },
                };
                *player_spawn_position = SpawnPosition(*player_position);
                let mut penguin_spawn_positions = vec![*player_position];
                let mut penguin_tags = vec![*player_penguin_tag];

                let translation = &mut transform.translation;
                translation.x = get_x(player_position.x);
                translation.y = get_y(player_position.y);

                // face the player right
                sprite.flip_x = false;

                // make the player temporarily immortal
                commands.entity(player_entity).insert(Immortal::default());

                let mob_spawn_positions = spawn_story_mode_mobs(
                    &mut commands,
                    &game_textures,
                    story_mode_context.level,
                    *world_id,
                    *map_size,
                );

                if let Level::BossRoom = story_mode_context.level {
                    let (boss_spawn_position, boss_penguin_tag) =
                        spawn_story_mode_boss(&mut commands, &game_textures, *world_id, *map_size);
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
                    &game_textures,
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
                p2.p1().single_mut().sections[0].value =
                    format_hud_time(game_timer.0.duration().as_secs_f32().ceil() as usize);

                story_mode_context.level_outcome = None;

                if game_option_store.get(GameOption::Transition) {
                    story_mode_context.state = StoryModeState::MapTransition;
                    commands.insert_resource(MapTransitionInput {
                        wall_entity_reveal_groups,
                        next_state: AppState::StoryModeManager,
                    });
                    next_state.set(AppState::MapTransition);
                } else if let Level::BossRoom = story_mode_context.level {
                    story_mode_context.state = StoryModeState::BossSpeech;
                    next_state.set(AppState::BossSpeech);
                } else {
                    story_mode_context.state = StoryModeState::InGame;
                    next_state.set(AppState::StoryModeInGame);
                }
            }
            StoryModeState::MapTransition => {
                if let Level::BossRoom = story_mode_context.level {
                    story_mode_context.state = StoryModeState::BossSpeech;
                    next_state.set(AppState::BossSpeech);
                } else {
                    story_mode_context.state = StoryModeState::InGame;
                    next_state.set(AppState::StoryModeInGame);
                }
            }
            StoryModeState::BossSpeech => {
                story_mode_context.state = StoryModeState::InGame;
                next_state.set(AppState::StoryModeInGame);
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
                        p2.p0().single_mut().sections[0].value = format_hud_points(game_score.0);

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
                                *p.p2().single_mut() =
                                    hud_colors.get_background_color(*world_id).into();
                                game_textures.set_map_textures(*world_id);
                            }
                            (Level::Regular(num), _) => {
                                if num < 4 {
                                    story_mode_context.level = Level::Regular(num + 1);
                                } else {
                                    story_mode_context.level = Level::BossRoom
                                }
                            }
                        };

                        let mut tmp = p.p0();
                        let (player_entity, mut player_texture, base_texture, mut bomb_satchel) =
                            tmp.single_mut();

                        // reset the player's texture (clears immortality animation effects)
                        *player_texture = base_texture.0.clone();

                        let unexploded_player_bombs = query3
                            .iter()
                            .filter(|b| matches!(b.owner, Some(entity) if entity == player_entity))
                            .count();

                        for entity in query2.iter() {
                            commands.entity(entity).despawn();
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
                    next_state.set(AppState::HighScoreNameInput);
                } else {
                    // skip to the step below where we choose the next state
                    continue;
                }
            }
            StoryModeState::HighScoreNameInput => {
                next_state.set(AppState::StoryModeTeardown);
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
    if let Ok(player) = query2.get_single() {
        query.single_mut().sections[0].value = format_hud_lives(player.lives);
    }
}

pub fn hud_points_indicator_update(
    game_score: Res<GameScore>,
    mut query: Query<&mut Text, With<BottomLeftDisplay2>>,
) {
    if game_score.is_changed() {
        query.single_mut().sections[0].value = format_hud_points(game_score.0);
    }
}

pub fn finish_level(
    mut story_mode_context: ResMut<StoryModeContext>,
    game_timer: Res<GameTimer>,
    mut p: ParamSet<(
        Query<(&mut Position, &TeamID), (With<Player>, With<Protagonist>)>,
        Query<&Position, With<Exit>>,
    )>,
    query: Query<&Protagonist>,
    query2: Query<&TeamID, With<Player>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let mut level_outcome = None;

    match story_mode_context.level {
        Level::Regular(_) => {
            // if an exit is spawned...
            if let Ok(exit_position) = p.p1().get_single().map(|p| *p) {
                // ...check if a protagonist reached it when all the enemies are dead
                if p.p0().iter_mut().any(|(pp, ptid)| {
                    *pp == exit_position && !query2.iter().any(|tid| tid.0 != ptid.0)
                }) {
                    level_outcome = Some(LevelOutcome::Win);
                }
            }
        }
        Level::BossRoom => {
            // if a protagonist killed all the enemies
            if p.p0()
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

    if level_outcome.is_some() {
        story_mode_context.level_outcome = level_outcome;
        next_state.set(AppState::StoryModeManager);
    }
}

pub fn setup_boss_speech(
    mut commands: Commands,
    hud_colors: Res<HUDColors>,
    game_textures: Res<GameTextures>,
    boss_speech_script: Res<BossSpeechScript>,
    fonts: Res<Fonts>,
    query: Query<Entity, With<HUDRoot>>,
) {
    let mut speech_box = None;
    let mut speaker_portrait = None;
    let mut speech_text = None;

    commands.entity(query.single()).with_children(|parent| {
        speech_box = Some(
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            top: Val::Px(0.0),
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..Default::default()
                        },
                        background_color: hud_colors.black_color.into(),
                        ..Default::default()
                    },
                    UIComponent,
                ))
                .with_children(|parent| {
                    // dialog border
                    parent.spawn((
                        TextBundle {
                            text: Text::from_section(
                                r"
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                        │
│                                                                                        │
│                                                                                        │
│                                                                                        │
│                                                                                        │
└────────────────────────────────────────────────────────────────────────────────────────┘
"
                                .trim_matches('\n'),
                                TextStyle {
                                    font: fonts.mono.clone(),
                                    font_size: 2.0 * PIXEL_SCALE as f32,
                                    color: COLORS[15].into(), // TODO: is this the right color?
                                },
                            ),
                            style: Style {
                                position_type: PositionType::Absolute,
                                left: Val::Px(0.0),
                                top: Val::Px(0.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        UIComponent,
                    ));

                    // player portrait
                    parent
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(4.0 * PIXEL_SCALE as f32),
                                    top: Val::Px(2.0 * PIXEL_SCALE as f32),
                                    width: Val::Px(8.0 * PIXEL_SCALE as f32),
                                    height: Val::Px(10.0 * PIXEL_SCALE as f32),
                                    border: UiRect {
                                        left: Val::Px(PIXEL_SCALE as f32),
                                        top: Val::Px(PIXEL_SCALE as f32),
                                        right: Val::Px(PIXEL_SCALE as f32),
                                        bottom: Val::Px(PIXEL_SCALE as f32),
                                    },
                                    ..Default::default()
                                },
                                background_color: hud_colors.portrait_border_color.into(),
                                ..Default::default()
                            },
                            UIComponent,
                        ))
                        .with_children(|parent| {
                            parent
                                .spawn((
                                    NodeBundle {
                                        style: Style {
                                            width: Val::Percent(100.0),
                                            height: Val::Percent(100.0),
                                            ..Default::default()
                                        },
                                        background_color: hud_colors
                                            .portrait_background_color
                                            .into(),
                                        ..Default::default()
                                    },
                                    UIComponent,
                                ))
                                .with_children(|parent| {
                                    speaker_portrait = Some(
                                        parent
                                            .spawn((
                                                ImageBundle {
                                                    style: Style {
                                                        width: Val::Percent(100.0),
                                                        height: Val::Percent(100.0),
                                                        ..Default::default()
                                                    },
                                                    image: game_textures
                                                        .get_penguin_texture(
                                                            boss_speech_script
                                                                .get_current_speaker(),
                                                        )
                                                        .clone()
                                                        .into(),
                                                    ..Default::default()
                                                },
                                                UIComponent,
                                            ))
                                            .id(),
                                    );
                                });
                        });

                    // speech text
                    speech_text = Some(
                        parent
                            .spawn((
                                TextBundle {
                                    text: Text::from_section(
                                        boss_speech_script.get_current_line_state(),
                                        TextStyle {
                                            font: fonts.mono.clone(),
                                            font_size: 2.0 * PIXEL_SCALE as f32,
                                            color: COLORS[15].into(), // TODO: is this the right color?
                                        },
                                    ),
                                    style: Style {
                                        position_type: PositionType::Absolute,
                                        top: Val::Px(6.0 * PIXEL_SCALE as f32),
                                        left: Val::Px(16.0 * PIXEL_SCALE as f32),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                UIComponent,
                            ))
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
    game_textures: Res<GameTextures>,
    mut boss_speech_script: ResMut<BossSpeechScript>,
    boss_speech_box_entities: Res<BossSpeechBoxEntities>,
    inputs: Res<InputActionStatusTracker>,
    mut next_state: ResMut<NextState<AppState>>,
    mut query: Query<&mut Text>,
    mut query2: Query<&mut UiImage>,
) {
    boss_speech_script.tick(time.delta());

    if inputs.is_active(InputAction::Space) {
        if boss_speech_script.line_in_progress() {
            boss_speech_script.complete_current_line();
        } else if boss_speech_script.advance_script().is_ok() {
            *query2
                .get_mut(boss_speech_box_entities.speaker_portrait)
                .unwrap() = game_textures
                .get_penguin_texture(boss_speech_script.get_current_speaker())
                .clone()
                .into();
        } else {
            commands
                .entity(boss_speech_box_entities.speech_box)
                .despawn_recursive();

            commands.remove_resource::<BossSpeechBoxEntities>();
            commands.remove_resource::<BossSpeechScript>();

            next_state.set(AppState::StoryModeManager);
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
    hud_colors: Res<HUDColors>,
    fonts: Res<Fonts>,
    query: Query<Entity, With<UIRoot>>,
    map_size: Res<MapSize>,
) {
    let mut name_text = None;

    commands.entity(query.single()).with_children(|parent| {
        parent
            .spawn((
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Px(
                            ((map_size.columns * (TILE_WIDTH / PIXEL_SCALE) / 2 - 15) * PIXEL_SCALE)
                                as f32,
                        ),
                        top: Val::Px(
                            // messy equation that produces the same results as the C code (integer divisions)
                            ((((HUD_HEIGHT + map_size.rows * TILE_HEIGHT) / PIXEL_SCALE) / 4 * 2
                                - 6)
                                * PIXEL_SCALE) as f32,
                        ),
                        width: Val::Px(30.0 * PIXEL_SCALE as f32),
                        height: Val::Px(10.0 * PIXEL_SCALE as f32),
                        ..Default::default()
                    },
                    background_color: hud_colors.black_color.into(),
                    ..Default::default()
                },
                UIComponent,
            ))
            .with_children(|parent| {
                // dialog border
                parent.spawn((
                    TextBundle {
                        text: Text::from_section(
                            r"
┌────────────────────────────┐
│                            │
│ Name:                      │
│                            │
└────────────────────────────┘
"
                            .trim_matches('\n'),
                            TextStyle {
                                font: fonts.mono.clone(),
                                font_size: 2.0 * PIXEL_SCALE as f32,
                                color: COLORS[15].into(), // TODO: is this the right color?
                            },
                        ),
                        style: Style {
                            position_type: PositionType::Absolute,
                            top: Val::Px(0.0),
                            left: Val::Px(0.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    UIComponent,
                ));

                // name text
                name_text = Some(
                    parent
                        .spawn((
                            TextBundle {
                                text: Text::from_section(
                                    "",
                                    TextStyle {
                                        font: fonts.mono.clone(),
                                        font_size: 2.0 * PIXEL_SCALE as f32,
                                        color: COLORS[15].into(), // TODO: is this the right color?
                                    },
                                ),
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    top: Val::Px(4.0 * PIXEL_SCALE as f32),
                                    left: Val::Px(8.0 * PIXEL_SCALE as f32),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            UIComponent,
                        ))
                        .id(),
                );
            });
    });

    commands.insert_resource(HighScoreNameInputContext {
        name_text: name_text.unwrap(),
    });
}

pub fn high_score_name_input_update(
    mut commands: Commands,
    context: Res<HighScoreNameInputContext>,
    inputs: Res<InputActionStatusTracker>,
    mut char_input_events: EventReader<ReceivedCharacter>,
    mut persistent_high_scores: ResMut<PersistentHighScores>,
    game_score: Res<GameScore>,
    mut query: Query<&mut Text>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if inputs.is_active(InputAction::Escape) {
        persistent_high_scores.insert_score(String::from("<unnamed_player>"), game_score.0);
        commands.remove_resource::<HighScoreNameInputContext>();
        next_state.set(AppState::StoryModeManager);
        return;
    }

    let name = &mut query.get_mut(context.name_text).unwrap().sections[0].value;

    for event in char_input_events.read() {
        if name.len() < 20 && (event.char == ' ' || event.char.is_ascii_graphic()) {
            name.push(if event.char == ' ' { '_' } else { event.char });
        }
    }

    if inputs.is_active(InputAction::Back) {
        name.pop();
    }

    if inputs.is_active(InputAction::Return) {
        let name = if name.is_empty() {
            String::from("<unnamed_player>")
        } else {
            name.clone()
        };

        persistent_high_scores.insert_score(name, game_score.0);
        commands.remove_resource::<HighScoreNameInputContext>();
        next_state.set(AppState::StoryModeManager);
    }
}

pub fn teardown(
    mut commands: Commands,
    query: Query<Entity, Without<Window>>,
    story_mode_context: Res<StoryModeContext>,
    mut player_action_events: ResMut<Events<PlayerActionEvent>>,
    mut explosion_events: ResMut<Events<ExplosionEvent>>,
    mut burn_events: ResMut<Events<BurnEvent>>,
    mut damage_events: ResMut<Events<DamageEvent>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // clear entities
    for entity in query.iter() {
        commands.entity(entity).despawn();
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

    if story_mode_context.game_completed {
        next_state.set(AppState::SecretModeSetup);
    } else {
        next_state.set(AppState::MainMenu);
    };
}
