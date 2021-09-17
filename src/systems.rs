use std::{
    collections::{HashMap, HashSet},
    iter::Iterator,
    time::Duration,
};

use bevy::{
    app::{AppExit, Events},
    prelude::*,
    render::camera::{Camera, VisibleEntities},
};
use bevy_kira_audio::Audio;
use rand::{prelude::*, Rng};

use crate::{
    camera::*,
    components::*,
    constants::*,
    events::*,
    item::*,
    resources::*,
    types::{Direction, *},
    utils::*,
    AppState,
};

pub fn set_volume_based_on_options(game_option_store: Res<GameOptionStore>, audio: Res<Audio>) {
    if game_option_store.get(GameOption::Sound) {
        audio.set_volume(1.0);
    } else {
        audio.set_volume(0.0);
    }
}

pub fn setup_splash_screen(
    mut commands: Commands,
    base_color_materials: Res<BaseColorMaterials>,
    fonts: Res<Fonts>,
) {
    commands.spawn_bundle(UiCameraBundle::default());

    let right_position = MENU_WIDTH - SPLASH_SCREEN_TEXT_RIGHT.len() * PIXEL_SCALE;

    let mut left_text = None;
    let mut right_text = None;
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                ..Default::default()
            },
            material: base_color_materials.colors[0].clone(),
            ..Default::default()
        })
        .insert(UIRoot)
        .insert(UIComponent)
        .with_children(|parent| {
            left_text = Some(
                parent
                    .spawn_bundle(TextBundle {
                        text: Text::with_section(
                            SPLASH_SCREEN_TEXT_LEFT,
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
                                top: Val::Px((MENU_HEIGHT / 2) as f32),
                                left: Val::Px(0.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(UIComponent)
                    .id(),
            );

            right_text = Some(
                parent
                    .spawn_bundle(TextBundle {
                        text: Text::with_section(
                            SPLASH_SCREEN_TEXT_RIGHT,
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
                                top: Val::Px((MENU_HEIGHT / 2 + 2 * PIXEL_SCALE) as f32),
                                left: Val::Px(right_position as f32),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(UIComponent)
                    .id(),
            );
        });

    commands.insert_resource(SplashScreenContext {
        left_text: left_text.unwrap(),
        right_text: right_text.unwrap(),
        left_position: 0,
        right_position,
        text_state: SplashScreenTextState::Moving(Timer::from_seconds(0.03, true)),
    })
}

pub fn splash_screen_update(
    mut commands: Commands,
    time: Res<Time>,
    mut splash_screen_context: ResMut<SplashScreenContext>,
    mut query: Query<&mut Style>,
    mut state: ResMut<State<AppState>>,
) {
    match splash_screen_context.text_state {
        SplashScreenTextState::Moving(ref mut timer) => {
            const LEFT_END_POSITION: usize = MENU_WIDTH / 2 - 5 * PIXEL_SCALE;
            const RIGHT_END_POSITION: usize =
                MENU_WIDTH / 2 - (SPLASH_SCREEN_TEXT_RIGHT.len() - 5) * PIXEL_SCALE;

            timer.tick(time.delta());
            if timer.finished() {
                let times_finished = timer.times_finished();

                splash_screen_context.left_position = (splash_screen_context.left_position
                    + times_finished as usize * PIXEL_SCALE)
                    .min(LEFT_END_POSITION);
                query
                    .get_mut(splash_screen_context.left_text)
                    .unwrap()
                    .position
                    .left = Val::Px(splash_screen_context.left_position as f32);

                splash_screen_context.right_position = splash_screen_context
                    .right_position
                    .saturating_sub(times_finished as usize * PIXEL_SCALE)
                    .max(RIGHT_END_POSITION);
                query
                    .get_mut(splash_screen_context.right_text)
                    .unwrap()
                    .position
                    .left = Val::Px(splash_screen_context.right_position as f32);

                if splash_screen_context.left_position == LEFT_END_POSITION
                    && splash_screen_context.right_position == RIGHT_END_POSITION
                {
                    splash_screen_context.text_state =
                        SplashScreenTextState::Holding(Timer::from_seconds(0.7, false));
                }
            }
        }
        SplashScreenTextState::Holding(ref mut timer) => {
            timer.tick(time.delta());
            if timer.finished() {
                commands.remove_resource::<SplashScreenContext>();
                state.set(AppState::MainMenu).unwrap();
            }
        }
    }
}

pub fn setup_menu(
    fonts: Res<Fonts>,
    mut commands: Commands,
    menu_materials: Res<MenuMaterials>,
    menu_state: Res<MenuState>,
    game_option_store: Res<GameOptionStore>,
    persistent_high_scores: Res<PersistentHighScores>,
) {
    commands.spawn_bundle(UiCameraBundle::default());

    let mut menu_background_animation_context = None;
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                ..Default::default()
            },
            material: menu_materials.background.clone(),
            ..Default::default()
        })
        .insert(UIRoot)
        .insert(UIComponent)
        .with_children(|parent| {
            let title_text = r#"
 ____   ____  __  __ ____  ______ _____  __  __          _   _ 
|  _ \ / __ \|  \/  |  _ \|  ____|  __ \|  \/  |   /\   | \ | |
| |_) | |  | | \  / | |_) | |__  | |__) | \  / |  /  \  |  \| |
|  _ <| |  | | |\/| |  _ <|  __| |  _  /| |\/| | / /\ \ | . ` |
| |_) | |__| | |  | | |_) | |____| | \ \| |  | |/ ____ \| |\  |
|____/ \____/|_|  |_|____/|______|_|  \_\_|  |_/_/    \_\_| \_|
"#;
            parent
                .spawn_bundle(TextBundle {
                    text: Text::with_section(
                        title_text.to_string(),
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
                            top: Val::Px(12.0 * PIXEL_SCALE as f32),
                            left: Val::Px(17.0 * PIXEL_SCALE as f32),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(UIComponent);

            let mut place_text = |y, x, str: &str, c: usize| {
                parent
                    .spawn_bundle(TextBundle {
                        text: Text::with_section(
                            str.to_string(),
                            TextStyle {
                                font: fonts.mono.clone(),
                                font_size: 2.0 * PIXEL_SCALE as f32,
                                color: COLORS[c].into(),
                            },
                            TextAlignment::default(),
                        ),
                        style: Style {
                            position_type: PositionType::Absolute,
                            position: Rect {
                                top: Val::Px(y as f32 * 2.0 * PIXEL_SCALE as f32),
                                left: Val::Px(x as f32 * PIXEL_SCALE as f32),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(UIComponent)
                    .id()
            };

            let c = 14;
            place_text(10, 4, "*", c);
            place_text(30, 9, "*", c);
            place_text(46, 70, "*", c);
            place_text(44, 5, "*", c);
            place_text(17, 72, "*", c);
            place_text(
                39,
                83,
                r#"
 \__/
  ██
__██__
  ||
 =██=
  ||
"#,
                8,
            );

            let mut entity_change_parameters = HashMap::new();

            let mut add_text_entity_change_parameters = |y, x, modulo, threshold, v1, v2| {
                let initial_value = String::from(v1);
                entity_change_parameters.insert(
                    place_text(y, x, &initial_value, c),
                    MenuBackgroundEntityChangeParameters {
                        modulo,
                        threshold,
                        values: MenuBackgroundEntityValues::Text(initial_value, String::from(v2)),
                    },
                );
            };

            add_text_entity_change_parameters(15, 2, 200, 100, "*", "\u{2219}");
            add_text_entity_change_parameters(40, 21, 100, 50, "\u{2219}", "*");
            add_text_entity_change_parameters(43, 40, 222, 111, "*", "+");
            add_text_entity_change_parameters(30, 86, 700, 350, "*", "\u{2219}");
            add_text_entity_change_parameters(5, 91, 312, 156, "*", "+");
            add_text_entity_change_parameters(13, 78, 160, 80, " ", "*");
            add_text_entity_change_parameters(19, 92, 123, 62, "\u{2219}", "*");

            // add_color_entity_change_parameters
            let initial_value = 4;
            entity_change_parameters.insert(
                place_text(
                    38,
                    82,
                    r#"
 .    .


.      .
"#,
                    initial_value,
                ),
                MenuBackgroundEntityChangeParameters {
                    modulo: 348,
                    threshold: 174,
                    values: MenuBackgroundEntityValues::Color(
                        COLORS[initial_value].into(),
                        COLORS[12].into(),
                    ),
                },
            );

            menu_background_animation_context = Some(MenuBackgroundAnimationContext {
                entity_change_parameters,
                timer: Timer::from_seconds(100.0, true),
            });

            // spawn central modal
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(
                            Val::Px(40.0 * PIXEL_SCALE as f32),
                            Val::Px(40.0 * PIXEL_SCALE as f32),
                        ),
                        position_type: PositionType::Absolute,
                        position: Rect {
                            left: Val::Px(30.0 * PIXEL_SCALE as f32),
                            top: Val::Px(36.0 * PIXEL_SCALE as f32),
                            ..Default::default()
                        },
                        border: Rect {
                            left: Val::Px(PIXEL_SCALE as f32),
                            top: Val::Px(2.0 * PIXEL_SCALE as f32),
                            right: Val::Px(PIXEL_SCALE as f32),
                            bottom: Val::Px(2.0 * PIXEL_SCALE as f32),
                        },
                        ..Default::default()
                    },
                    material: menu_materials.modal_foreground.clone(),
                    ..Default::default()
                })
                .insert(UIComponent)
                .with_children(|parent| {
                    // spawn modal border
                    parent
                        .spawn_bundle(TextBundle {
                            text: Text::with_section(
                                r#"
┌──────────────────────────────────────┐
│                                      │
│                                      │
│                                      │
│                                      │
│                                      │
│                                      │
│                                      │
│                                      │
│                                      │
│                                      │
│                                      │
│                                      │
│                                      │
│                                      │
│                                      │
│                                      │
│                                      │
│                                      │
└──────────────────────────────────────┘
"#,
                                TextStyle {
                                    font: fonts.mono.clone(),
                                    font_size: 2.0 * PIXEL_SCALE as f32,
                                    color: menu_materials.modal_background_color,
                                },
                                TextAlignment::default(),
                            ),
                            style: Style {
                                position_type: PositionType::Absolute,
                                position: Rect {
                                    top: Val::Px(-2.0 * PIXEL_SCALE as f32),
                                    left: Val::Px(-1.0 * PIXEL_SCALE as f32),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(UIComponent);

                    // spawn menu type
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                ..Default::default()
                            },
                            material: menu_materials.modal_background.clone(),
                            ..Default::default()
                        })
                        .insert(UIComponent)
                        .insert(MenuContentBox)
                        .with_children(|parent| {
                            spawn_menu_type(
                                parent,
                                menu_state.get_current_menu(),
                                &fonts,
                                &menu_materials,
                                &game_option_store,
                                persistent_high_scores.get_raw_scores(),
                            );
                        });
                });
        });

    commands.insert_resource(menu_background_animation_context.unwrap());

    if game_option_store.get(GameOption::Demo) {
        commands.insert_resource(DemoModeStartTimer(Timer::from_seconds(
            DEMO_MODE_START_TIMER_DURATION_SECS,
            false,
        )));
    }
}

pub fn menu_navigation(
    mut commands: Commands,
    audio: Res<Audio>,
    sounds: Res<Sounds>,
    fonts: Res<Fonts>,
    menu_materials: Res<MenuMaterials>,
    mut state: ResMut<State<AppState>>,
    mut menu_state: ResMut<MenuState>,
    mut game_option_store: ResMut<GameOptionStore>,
    persistent_high_scores: Res<PersistentHighScores>,
    mut keyboard_input: ResMut<Input<KeyCode>>,
    mut query: Query<(Entity, &Children), With<MenuContentBox>>,
    mut query3: Query<(Entity, &Children), With<BattleModeSubMenuContentBox>>,
    mut ev_exit: EventWriter<AppExit>,
) {
    let mut menu_changed = false;
    if let Some(ref mut sub_menu_state) = menu_state.battle_mode_sub_menu_state {
        if keyboard_input.just_pressed(KeyCode::Left) {
            match sub_menu_state.step {
                BattleModeSubMenuStep::AmountOfPlayers => {
                    sub_menu_state.amount_of_players.decrement()
                }
                BattleModeSubMenuStep::AmountOfBots => sub_menu_state.amount_of_bots.decrement(),
                BattleModeSubMenuStep::WinningScore => sub_menu_state.winning_score.decrement(),
                BattleModeSubMenuStep::Difficulty => sub_menu_state.difficulty.decrement(),
            }
            menu_changed = true;
        }

        if keyboard_input.just_pressed(KeyCode::Right) {
            match sub_menu_state.step {
                BattleModeSubMenuStep::AmountOfPlayers => {
                    sub_menu_state.amount_of_players.increment()
                }
                BattleModeSubMenuStep::AmountOfBots => sub_menu_state.amount_of_bots.increment(),
                BattleModeSubMenuStep::WinningScore => sub_menu_state.winning_score.increment(),
                BattleModeSubMenuStep::Difficulty => sub_menu_state.difficulty.increment(),
            }
            menu_changed = true;
        }

        if keyboard_input.just_pressed(KeyCode::Return) {
            match sub_menu_state.step {
                BattleModeSubMenuStep::AmountOfPlayers => {
                    sub_menu_state.step = BattleModeSubMenuStep::AmountOfBots
                }
                BattleModeSubMenuStep::AmountOfBots => {
                    sub_menu_state.step = BattleModeSubMenuStep::WinningScore
                }
                BattleModeSubMenuStep::WinningScore => {
                    sub_menu_state.step = BattleModeSubMenuStep::Difficulty
                }
                BattleModeSubMenuStep::Difficulty => {
                    commands.insert_resource(BattleModeConfiguration {
                        amount_of_players: *sub_menu_state.amount_of_players.value(),
                        amount_of_bots: *sub_menu_state.amount_of_bots.value(),
                        winning_score: *sub_menu_state.winning_score.value(),
                        difficulty: *sub_menu_state.difficulty.value(),
                    });

                    menu_state.battle_mode_sub_menu_state = None;
                    state.push(AppState::BattleMode).unwrap();
                    keyboard_input.reset(KeyCode::Return);
                    return;
                }
            }
            menu_changed = true;
        }

        if keyboard_input.just_pressed(KeyCode::Escape) {
            menu_state.battle_mode_sub_menu_state = None;
            menu_changed = true;
        }
    } else {
        if keyboard_input.just_pressed(KeyCode::Return) {
            audio.play(sounds.confirm.clone());
            match menu_state.get_enter_action() {
                MenuAction::SwitchMenu(menu_id) => {
                    menu_state.switch_menu(menu_id);
                    menu_changed = true;
                }
                MenuAction::LaunchStoryMode => {
                    state.push(AppState::StoryMode).unwrap();
                    keyboard_input.reset(KeyCode::Return);
                    return;
                }
                MenuAction::OpenBattleModeSubMenu => {
                    let sub_menu_state = BattleModeSubMenuState::default();
                    commands
                        .entity(query.single().unwrap().0)
                        .with_children(|parent| {
                            spawn_battle_mode_sub_menu_modal(
                                parent,
                                &sub_menu_state,
                                &fonts,
                                &menu_materials,
                            );
                        });
                    menu_state.battle_mode_sub_menu_state = Some(sub_menu_state);
                    return;
                }
                MenuAction::ToggleOption(option) => {
                    let option_enabled = game_option_store.toggle(option);
                    menu_changed = true;

                    match option {
                        GameOption::Sound => {
                            if option_enabled {
                                audio.set_volume(1.0);
                            } else {
                                audio.set_volume(0.0);
                            }
                        }
                        GameOption::Demo => {
                            if option_enabled {
                                commands.insert_resource(DemoModeStartTimer(Timer::from_seconds(
                                    DEMO_MODE_START_TIMER_DURATION_SECS,
                                    false,
                                )));
                            } else {
                                commands.remove_resource::<DemoModeStartTimer>();
                            }
                        }
                        // TODO: implement
                        GameOption::Transition => (),
                    }
                }
                MenuAction::Exit => {
                    ev_exit.send(AppExit);
                    return;
                }
                MenuAction::Back => {
                    if menu_state.back().is_ok() {
                        menu_changed = true;
                    }
                }
            }
        }

        if keyboard_input.just_pressed(KeyCode::Escape) && menu_state.back().is_ok() {
            menu_changed = true;
        }

        if keyboard_input.just_pressed(KeyCode::Down) {
            match menu_state.get_current_menu_mut() {
                MenuType::SelectableItems(selectable_items) => {
                    audio.play(sounds.select.clone());
                    selectable_items.cycle_cursor_up();
                    menu_changed = true;
                }
                MenuType::ToggleableOptions(toggleable_options) => {
                    audio.play(sounds.select.clone());
                    toggleable_options.cycle_cursor_up();
                    menu_changed = true;
                }
                MenuType::StaticText(_) | MenuType::ControlsScreen(_) | MenuType::HallOfFame => (),
            }
        }

        if keyboard_input.just_pressed(KeyCode::Up) {
            match menu_state.get_current_menu_mut() {
                MenuType::SelectableItems(selectable_items) => {
                    audio.play(sounds.select.clone());
                    selectable_items.cycle_cursor_down();
                    menu_changed = true;
                }
                MenuType::ToggleableOptions(toggleable_options) => {
                    audio.play(sounds.select.clone());
                    toggleable_options.cycle_cursor_down();
                    menu_changed = true;
                }
                MenuType::StaticText(_) | MenuType::ControlsScreen(_) | MenuType::HallOfFame => (),
            }
        }

        if let MenuType::ControlsScreen(_) = menu_state.get_current_menu() {
            if keyboard_input.just_pressed(KeyCode::F) {
                state.push(AppState::SecretMode).unwrap();
                keyboard_input.reset(KeyCode::F);
                return;
            }
        }
    }

    if menu_changed {
        if let Some(sub_menu_state) = &menu_state.battle_mode_sub_menu_state {
            // refresh sub menu
            let (entity, children) = query3.single_mut().unwrap();
            for child in children.iter() {
                commands.entity(*child).despawn_recursive();
            }
            commands.entity(entity).with_children(|parent| {
                spawn_battle_mode_sub_menu_content(parent, sub_menu_state, &fonts, &menu_materials);
            });
        } else {
            // refresh main menu
            let (entity, children) = query.single_mut().unwrap();
            for child in children.iter() {
                commands.entity(*child).despawn_recursive();
            }
            commands.entity(entity).with_children(|parent| {
                spawn_menu_type(
                    parent,
                    menu_state.get_current_menu(),
                    &fonts,
                    &menu_materials,
                    &game_option_store,
                    persistent_high_scores.get_raw_scores(),
                );
            });
        }
    }
}

pub fn menu_demo_mode_trigger(
    mut commands: Commands,
    time: Res<Time>,
    demo_mode_start_timer: Option<ResMut<DemoModeStartTimer>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut state: ResMut<State<AppState>>,
) {
    if let Some(mut demo_mode_start_timer) = demo_mode_start_timer {
        if keyboard_input.get_just_pressed().len() > 0 {
            demo_mode_start_timer.0.reset();
        } else {
            demo_mode_start_timer.0.tick(time.delta());
            if demo_mode_start_timer.0.finished() {
                // state switching should fail here if there's a manually triggered state already queued
                if state.push(AppState::BattleMode).is_ok() {
                    println!("Starting demo mode!");

                    commands.insert_resource(BattleModeConfiguration {
                        amount_of_players: 0,
                        amount_of_bots: 8,
                        winning_score: 1,
                        difficulty: BotDifficulty::Medium,
                    });
                }
            }
        }
    }
}

pub fn animate_menu_background(
    time: Res<Time>,
    mut menu_background_animation_context: ResMut<MenuBackgroundAnimationContext>,
    mut query: Query<&mut Text>,
) {
    menu_background_animation_context.timer.tick(time.delta());
    let k = (menu_background_animation_context.timer.percent() * 3000.0) as usize;

    for (entity, change_parameters) in &menu_background_animation_context.entity_change_parameters {
        match &change_parameters.values {
            MenuBackgroundEntityValues::Text(v1, v2) => {
                query.get_mut(*entity).unwrap().sections[0].value =
                    if k % change_parameters.modulo < change_parameters.threshold {
                        v1.clone()
                    } else {
                        v2.clone()
                    }
            }
            MenuBackgroundEntityValues::Color(v1, v2) => {
                query.get_mut(*entity).unwrap().sections[0].style.color =
                    if k % change_parameters.modulo < change_parameters.threshold {
                        *v1
                    } else {
                        *v2
                    };
            }
        }
    }
}

pub fn resize_window(
    mut windows: ResMut<Windows>,
    state: Res<State<AppState>>,
    map_size: Option<Res<MapSize>>,
) {
    let window = windows.get_primary_mut().unwrap();
    match state.current() {
        AppState::StoryMode | AppState::BattleMode | AppState::SecretMode => {
            let map_size = map_size.unwrap();
            window.set_resolution(
                (map_size.columns * TILE_WIDTH) as f32,
                (HUD_HEIGHT + map_size.rows * TILE_HEIGHT) as f32,
            );
        }
        AppState::MainMenu => window.set_resolution(MENU_WIDTH as f32, MENU_HEIGHT as f32),
        AppState::SplashScreen
        | AppState::MapTransition
        | AppState::BossSpeech
        | AppState::HighScoreNameInput
        | AppState::LeaderboardDisplay
        | AppState::Paused => (),
    }
}

pub fn setup_story_mode(
    mut commands: Commands,
    mut textures: ResMut<Textures>,
    base_color_materials: Res<BaseColorMaterials>,
    hud_materials: Res<HUDMaterials>,
    fonts: Res<Fonts>,
    game_option_store: Res<GameOptionStore>,
    mut state: ResMut<State<AppState>>,
) {
    let map_size = MapSize {
        rows: 11,
        columns: 15,
    };
    let world_id = WorldID(1);
    let level = Level::Regular(1);
    let player_lives = 2;
    let player_points = 0;
    const LEVEL_DURATION_SECONDS: usize = 180;

    textures.set_map_textures(world_id);

    // spawn camera
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

    // spawn mobs
    let mob_spawn_positions =
        spawn_story_mode_mobs(&mut commands, &textures, level, world_id, map_size);

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
                &textures,
                world_id,
                &[player_penguin_tag],
                *state.current(),
                Some(player_lives),
                Some(player_points),
                Some(LEVEL_DURATION_SECONDS),
            );
        });

    let wall_entity_reveal_groups = spawn_map(
        &mut commands,
        &textures,
        map_size,
        if let Level::BossRoom = level {
            0.0
        } else {
            50.0
        },
        true,
        &[player_spawn_position],
        &mob_spawn_positions,
        matches!(level, Level::Regular(_)),
    );

    commands.insert_resource(GameScore(player_points));
    commands.insert_resource(GameTimer(Timer::from_seconds(
        LEVEL_DURATION_SECONDS as f32,
        false,
    )));
    commands.insert_resource(level);
    commands.insert_resource(world_id);
    commands.insert_resource(map_size);

    if game_option_store.get(GameOption::Transition) {
        commands.insert_resource(MapTransitionInput {
            wall_entity_reveal_groups,
        });
        state.push(AppState::MapTransition).unwrap();
    }
}

pub fn setup_battle_mode(
    mut commands: Commands,
    mut textures: ResMut<Textures>,
    fonts: Res<Fonts>,
    base_color_materials: Res<BaseColorMaterials>,
    hud_materials: Res<HUDMaterials>,
    battle_mode_configuration: Res<BattleModeConfiguration>,
    game_option_store: Res<GameOptionStore>,
    mut state: ResMut<State<AppState>>,
) {
    const ROUND_DURATION_SECS: usize = 120;

    let world_id = WorldID(rand::thread_rng().gen_range(1..=3));
    textures.set_map_textures(world_id);

    let (map_size, percent_of_passable_positions_to_fill) = get_battle_mode_map_size_fill(
        battle_mode_configuration.amount_of_players + battle_mode_configuration.amount_of_bots,
    );

    // spawn camera
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

    // map generation //
    let (player_spawn_positions, penguin_tags) = spawn_battle_mode_players(
        &mut commands,
        &textures,
        map_size,
        battle_mode_configuration.amount_of_players,
        battle_mode_configuration.amount_of_bots,
    );

    let wall_entity_reveal_groups = spawn_map(
        &mut commands,
        &textures,
        map_size,
        percent_of_passable_positions_to_fill,
        true,
        &player_spawn_positions,
        &[],
        false,
    );

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
                &textures,
                world_id,
                &penguin_tags,
                *state.current(),
                None,
                None,
                Some(ROUND_DURATION_SECS),
            );
        });

    commands.insert_resource(Leaderboard {
        scores: penguin_tags.into_iter().map(|p| (p, 0)).collect(),
        last_round_winner: None,
        winning_score: battle_mode_configuration.winning_score,
    });
    commands.insert_resource(GameTimer(Timer::from_seconds(
        ROUND_DURATION_SECS as f32,
        false,
    )));
    commands.insert_resource(WallOfDeath::Dormant(Timer::from_seconds(
        ROUND_DURATION_SECS as f32 / 2.0,
        false,
    )));
    commands.insert_resource(world_id);
    commands.insert_resource(map_size);

    if game_option_store.get(GameOption::Transition) {
        commands.insert_resource(MapTransitionInput {
            wall_entity_reveal_groups,
        });
        state.push(AppState::MapTransition).unwrap();
    }
}

pub fn setup_map_transition(
    mut commands: Commands,
    mut map_spawn_input: ResMut<MapTransitionInput>,
    mut query: Query<
        &mut Visible,
        Or<(
            With<Wall>,
            With<Player>,
            With<PenguinPortrait>,
            With<PenguinPortraitChild>,
        )>,
    >,
) {
    // hide wall, player and portrait entities
    for mut visible in query.iter_mut() {
        visible.is_visible = false;
    }

    commands.insert_resource(MapTransitionContext {
        wall_entity_reveal_groups: map_spawn_input
            .wall_entity_reveal_groups
            .drain(..)
            .collect(),
        reveal_timer: Timer::from_seconds(0.015, true),
    });
    commands.remove_resource::<MapTransitionInput>();
}

pub fn map_transition_update(
    mut commands: Commands,
    time: Res<Time>,
    mut map_transition_context: ResMut<MapTransitionContext>,
    level: Option<Res<Level>>,
    mut state: ResMut<State<AppState>>,
    mut query: Query<&mut Visible>,
) {
    let mut transition_done = false;
    // TODO: why is the first tick much larger? it progresses the transition further than we want
    map_transition_context.reveal_timer.tick(time.delta());
    for _ in 0..map_transition_context.reveal_timer.times_finished() {
        if let Some(reveal_group) = map_transition_context.wall_entity_reveal_groups.pop_front() {
            for entity in reveal_group {
                query.get_mut(entity).unwrap().is_visible = true;
            }
        } else {
            // reveal the rest of the hidden entities
            for mut visible in query.iter_mut() {
                visible.is_visible = true;
            }
            transition_done = true;
            break;
        }
    }

    if transition_done {
        // exit the map transition state
        if let Some(Level::BossRoom) = level.map(|v| *v) {
            state.set(AppState::BossSpeech).unwrap();
        } else {
            state.pop().unwrap();
        }
        commands.remove_resource::<MapTransitionContext>();
    }
}

pub fn hud_update(
    mut commands: Commands,
    game_score: Option<Res<GameScore>>,
    game_timer: Res<GameTimer>,
    mut q: QuerySet<(
        QueryState<&mut Text, With<LivesDisplay>>,
        QueryState<&mut Text, With<PointsDisplay>>,
        QueryState<&mut Text, With<GameTimerDisplay>>,
    )>,
    query3: Query<&Health, With<Protagonist>>,
    query4: Query<&Penguin>,
    query5: Query<(Entity, &PenguinPortrait)>,
    state: Res<State<AppState>>,
) {
    if matches!(state.current(), AppState::StoryMode)
        || state.inactives().contains(&AppState::StoryMode)
    {
        let mut tmp = q.q0();
        let mut text = tmp.single_mut().unwrap();
        text.sections[0].value = format!(
            "Lives:{}",
            if let Ok(player) = query3.single() {
                player.lives
            } else {
                // if a protagonist doesn't exist in story mode, that means he's dead == has 0 lives
                0
            }
        );
    }

    if let Some(game_score) = game_score {
        let mut tmp = q.q1();
        let mut text = tmp.single_mut().unwrap();
        text.sections[0].value = format!("Points:{}", game_score.0);
    }

    if let AppState::Paused = state.current() {
        q.q2().single_mut().unwrap().sections[0].value = String::from("PAUSE");
    } else {
        let remaining_seconds = (game_timer.0.duration() - game_timer.0.elapsed())
            .as_secs_f32()
            .ceil() as usize;
        q.q2().single_mut().unwrap().sections[0].value = format_hud_time(remaining_seconds);
    }

    // remove dead penguin portraits :(
    for (entity, PenguinPortrait(penguin)) in query5.iter() {
        if !query4.iter().any(|p| p.0 == penguin.0) {
            commands.entity(entity).despawn_recursive();
        }
    }
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
    game_score: Option<Res<GameScore>>,
    persistent_high_scores: Res<PersistentHighScores>,
    battle_mode_configuration: Option<Res<BattleModeConfiguration>>,
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

    if keyboard_input.just_pressed(KeyCode::Return)
        && !matches!(state.current(), AppState::SecretMode)
        && !matches!(
            battle_mode_configuration,
            Some(c) if c.amount_of_players == 0
        )
    {
        audio.stop();
        audio.play(sounds.pause.clone());
        state.push(AppState::Paused).unwrap();
        keyboard_input.reset(KeyCode::Return);
    }

    if keyboard_input.just_pressed(KeyCode::Escape) {
        if matches!(game_score, Some(game_score) if game_score.0 > persistent_high_scores.entry_threshold())
        {
            state.overwrite_push(AppState::HighScoreNameInput).unwrap();
        } else {
            state.overwrite_pop().unwrap();
        }
        keyboard_input.reset(KeyCode::Escape);
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

pub fn finish_level(
    mut commands: Commands,
    mut textures: ResMut<Textures>,
    hud_materials: Res<HUDMaterials>,
    game_option_store: Res<GameOptionStore>,
    mut level: ResMut<Level>,
    mut world_id: ResMut<WorldID>,
    map_size: Res<MapSize>,
    mut game_score: ResMut<GameScore>,
    mut game_timer: ResMut<GameTimer>,
    persistent_high_scores: Res<PersistentHighScores>,
    mut q: QuerySet<(
        QueryState<
            (
                Entity,
                &mut Position,
                &mut Transform,
                &mut Handle<ColorMaterial>,
                &BaseMaterial,
                &TeamID,
                &mut BombSatchel,
                &Penguin,
            ),
            (With<Player>, With<Protagonist>),
        >,
        QueryState<&Position, With<Exit>>,
        QueryState<&mut Handle<ColorMaterial>, With<HUDRoot>>,
    )>,
    query3: Query<&TeamID, With<Player>>,
    query4: Query<Entity, (Without<Camera>, Without<UIComponent>, Without<Protagonist>)>,
    // getting errors if these are split, possibly too many arguments
    mut q2: QuerySet<(
        QueryState<&Bomb>,
        QueryState<Entity, With<PenguinPortraitDisplay>>,
    )>,
    mut state: ResMut<State<AppState>>,
    mut keyboard_input: ResMut<Input<KeyCode>>,
) {
    let mut level_completed = false;
    match *level {
        Level::Regular(_) => {
            // if an exit is spawned...
            if let Ok(exit_position) = q.q1().single().map(|p| *p) {
                // ...check if a protagonist reached it when all the enemies are dead
                if q.q0().iter_mut().any(|(_, pp, _, _, _, ptid, _, _)| {
                    *pp == exit_position && !query3.iter().any(|tid| tid.0 != ptid.0)
                }) {
                    level_completed = true;
                }
            }
        }
        Level::BossRoom => {
            // if a protagonist killed all the enemies
            if q.q0()
                .iter_mut()
                .any(|(_, _, _, _, _, ptid, _, _)| !query3.iter().any(|tid| tid.0 != ptid.0))
            {
                level_completed = true;
            }
        }
    }

    // TODO: used for debugging, remove
    if keyboard_input.just_pressed(KeyCode::F) {
        level_completed = true;
        keyboard_input.reset(KeyCode::F);
    }

    if level_completed {
        if let Level::Regular(num) = *level {
            println!("Level {}x{} completed!", world_id.0, num);
        } else {
            println!("World {} boss defeated!", world_id.0);
        }

        // add 5 points for each second left on the clock
        game_score.0 += 5 * (game_timer.0.duration() - game_timer.0.elapsed()).as_secs() as usize;

        match (*level, world_id.0) {
            (Level::BossRoom, 3) => {
                game_score.0 += 2000;
                println!("Game completed! Final score: {}", game_score.0);

                if game_score.0 > persistent_high_scores.entry_threshold() {
                    state.overwrite_push(AppState::HighScoreNameInput).unwrap();
                } else {
                    state.overwrite_set(AppState::SecretMode).unwrap();
                }
                return;
            }
            (Level::BossRoom, _) => {
                world_id.0 += 1;
                *level = Level::Regular(1);
                *q.q2().single_mut().unwrap() =
                    hud_materials.get_background_material(*world_id).clone();
                textures.set_map_textures(*world_id);
            }
            (Level::Regular(num), _) => {
                if num < 4 {
                    *level = Level::Regular(num + 1);
                } else {
                    *level = Level::BossRoom
                }
            }
        }

        let mut tmp = q.q0();
        let (
            player_entity,
            mut player_position,
            mut transform,
            mut player_material,
            base_material,
            _,
            mut bomb_satchel,
            player_penguin_tag,
        ) = tmp.single_mut().unwrap();

        // reset the player's texture (clears immortality animation effects)
        *player_material = base_material.0.clone();

        let unexploded_player_bombs = q2
            .q0()
            .iter()
            .filter(|b| matches!(b.owner, Some(entity) if entity == player_entity))
            .count();

        for entity in query4.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // bomb refill
        bomb_satchel.bombs_available += unexploded_player_bombs;

        // move player to spawn
        *player_position = match *level {
            Level::Regular(_) => Position { y: 1, x: 1 },
            Level::BossRoom => Position {
                y: map_size.rows as isize - 4,
                x: map_size.columns as isize / 2,
            },
        };
        let mut penguin_spawn_positions = vec![*player_position];

        let translation = &mut transform.translation;
        translation.x = get_x(player_position.x);
        translation.y = get_y(player_position.y);

        // make temporarily immortal
        commands
            .entity(player_entity)
            .insert_bundle(ImmortalBundle::default());

        let mob_spawn_positions =
            spawn_story_mode_mobs(&mut commands, &textures, *level, *world_id, *map_size);

        if let Level::BossRoom = *level {
            let (boss_spawn_position, boss_penguin_tag) =
                spawn_story_mode_boss(&mut commands, &textures, *world_id, *map_size);
            penguin_spawn_positions.push(boss_spawn_position);

            // add enemy penguin portraits (the player's one is left as is)
            commands
                .entity(q2.q1().single().unwrap())
                .with_children(|parent| {
                    init_penguin_portraits(parent, &[boss_penguin_tag], &hud_materials, &textures);
                });

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
            if let Level::BossRoom = *level {
                0.0
            } else {
                50.0
            },
            true,
            &penguin_spawn_positions,
            &mob_spawn_positions,
            matches!(*level, Level::Regular(_)),
        );

        game_timer.0.reset();

        if game_option_store.get(GameOption::Transition) {
            commands.insert_resource(MapTransitionInput {
                wall_entity_reveal_groups,
            });
            state.push(AppState::MapTransition).unwrap();
        } else if let Level::BossRoom = *level {
            state.push(AppState::BossSpeech).unwrap();
        }
    }
}

pub fn fail_level(
    game_score: Res<GameScore>,
    game_timer: Res<GameTimer>,
    query: Query<&Protagonist>,
    mut state: ResMut<State<AppState>>,
) {
    if game_timer.0.finished() || query.iter().count() == 0 {
        println!("Game over! Final score: {}", game_score.0);
        state.overwrite_pop().unwrap();
    }
}

pub fn finish_round(
    mut commands: Commands,
    game_timer: Res<GameTimer>,
    mut leaderboard: ResMut<Leaderboard>,
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<&Penguin, With<Player>>,
    query2: Query<Entity, (Without<Camera>, Without<UIComponent>)>,
    query3: Query<Entity, With<PenguinPortrait>>,
    mut state: ResMut<State<AppState>>,
) {
    let mut round_over = false;
    if game_timer.0.finished() || query.iter().count() == 0 {
        println!("Round over with no winners!");
        leaderboard.last_round_winner = None;
        round_over = true;
    } else if let Ok(penguin) = query.single() {
        println!("Player {:?} won the round!", penguin.0);
        leaderboard.last_round_winner = Some(*penguin);
        *leaderboard.scores.get_mut(penguin).unwrap() += 1;
        round_over = true;
    }

    // TODO: used for debugging, remove
    if keyboard_input.just_pressed(KeyCode::F) {
        let winner_penguin = leaderboard
            .scores
            .iter()
            .choose(&mut rand::thread_rng())
            .map(|(p, _)| *p)
            .unwrap();

        leaderboard.last_round_winner = Some(winner_penguin);
        let score = leaderboard.scores.get_mut(&winner_penguin).unwrap();
        *score += 1;
        round_over = true;
    }

    if round_over {
        for entity in query2.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // clear penguin portraits
        for entity in query3.iter() {
            commands.entity(entity).despawn_recursive();
        }

        state.overwrite_push(AppState::LeaderboardDisplay).unwrap();
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
    mut query: Query<(
        Entity,
        &mut Perishable,
        &Position,
        Option<&Bomb>,
        Option<&Wall>,
    )>,
    mut query2: Query<&mut BombSatchel>,
    mut ev_explosion: EventWriter<ExplosionEvent>,
    state: Res<State<AppState>>,
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
                } else {
                    if rand::thread_rng().gen_range(0.0..1.0) < ITEM_SPAWN_CHANCE {
                        generate_item_at_position(*position, &mut commands, &textures, &state);
                    }
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
    mut game_score: Option<ResMut<GameScore>>,
    map_size: Res<MapSize>,
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
                            generate_item_at_position(position, &mut commands, &textures, &state);
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

pub fn wall_of_death_update(
    mut commands: Commands,
    time: Res<Time>,
    textures: Res<Textures>,
    mut wall_of_death: ResMut<WallOfDeath>,
    map_size: Res<MapSize>,
    query: Query<&Position, (With<Wall>, Without<Destructible>)>,
    query2: Query<(Entity, &Position, Option<&Bomb>)>,
    mut query3: Query<&mut BombSatchel>,
) {
    let get_next_position_direction = |mut position: Position,
                                       mut direction: Direction|
     -> Option<(Position, Direction)> {
        let end_position = Position {
            y: map_size.rows as isize - 3,
            x: 3,
        };

        let walls: HashSet<Position> = query.iter().copied().collect();
        loop {
            if position == end_position {
                break None;
            }

            match position {
                Position { y: 1, x: 1 } | Position { y: 2, x: 2 } => {
                    direction = Direction::Right;
                }
                Position { y: 1, x } if x == map_size.columns as isize - 2 => {
                    direction = Direction::Down;
                }
                Position { y, x }
                    if y == map_size.rows as isize - 2 && x == map_size.columns as isize - 2 =>
                {
                    direction = Direction::Left;
                }
                Position { y, x: 2 } if y == map_size.rows as isize - 2 => {
                    direction = Direction::Up;
                }
                Position { y: 2, x } if x == map_size.columns as isize - 3 => {
                    direction = Direction::Down;
                }
                Position { y, x }
                    if y == map_size.rows as isize - 3 && x == map_size.columns as isize - 3 =>
                {
                    direction = Direction::Left;
                }
                _ => (),
            }

            position = position.offset(direction, 1);
            if !walls.contains(&position) {
                break Some((position, direction));
            }
        }
    };

    let mut clear_position_and_spawn_wall = |position: Position| {
        for (e, _, b) in query2.iter().filter(|(_, p, _)| **p == position) {
            commands.entity(e).despawn_recursive();

            // TODO: this is the same logic as in perishable_tick, move into a separate system
            if let Some(bomb) = b {
                if let Some(owner) = bomb.owner {
                    if let Ok(mut bomb_satchel) = query3.get_mut(owner) {
                        bomb_satchel.bombs_available += 1;
                    }
                }
            }
        }

        commands
            .spawn_bundle(SpriteBundle {
                material: textures.get_map_textures().wall.clone(),
                transform: Transform::from_xyz(get_x(position.x), get_y(position.y), 0.0),
                sprite: Sprite::new(Vec2::new(TILE_WIDTH as f32, TILE_HEIGHT as f32)),
                ..Default::default()
            })
            .insert(Wall)
            .insert(Solid)
            .insert(position);
    };

    loop {
        let new_state = match *wall_of_death {
            WallOfDeath::Dormant(ref mut timer) => {
                timer.tick(time.delta());

                if timer.finished() {
                    println!("Wall of Death activated!");

                    Some(WallOfDeath::Active(ActiveWallOfDeath {
                        position: Position {
                            y: map_size.rows as isize - 1,
                            x: 1,
                        },
                        direction: Direction::Up,
                        cooldown: Cooldown::from_seconds(0.2),
                    }))
                } else {
                    None
                }
            }
            WallOfDeath::Active(ref mut active_wall_of_death) => {
                active_wall_of_death.cooldown.tick(time.delta());
                if active_wall_of_death.cooldown.ready() {
                    if let Some((position, direction)) = get_next_position_direction(
                        active_wall_of_death.position,
                        active_wall_of_death.direction,
                    ) {
                        active_wall_of_death.cooldown.trigger();
                        active_wall_of_death.position = position;
                        active_wall_of_death.direction = direction;

                        clear_position_and_spawn_wall(active_wall_of_death.position);

                        None
                    } else {
                        Some(WallOfDeath::Done)
                    }
                } else {
                    None
                }
            }
            WallOfDeath::Done => None,
        };

        if let Some(new_state) = new_state {
            *wall_of_death = new_state;
        } else {
            break;
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
    // menu
    commands.remove_resource::<DemoModeStartTimer>();
    commands.remove_resource::<MenuBackgroundAnimationContext>();

    // common
    commands.remove_resource::<GameTimer>();
    commands.remove_resource::<Level>();
    commands.remove_resource::<WorldID>();
    commands.remove_resource::<MapSize>();

    // story mode
    commands.remove_resource::<GameScore>();
    commands.remove_resource::<ExitPosition>();

    // battle mode
    commands.remove_resource::<Leaderboard>();
    commands.remove_resource::<WallOfDeath>();
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

pub fn setup_leaderboard_display(
    mut commands: Commands,
    base_color_materials: Res<BaseColorMaterials>,
    textures: Res<Textures>,
    fonts: Res<Fonts>,
    leaderboard: Res<Leaderboard>,
    windows: Res<Windows>,
    query: Query<Entity, With<UIRoot>>,
) {
    let mut leaderboard_display_box = None;
    let window = windows.get_primary().unwrap();

    commands
        .entity(query.single().unwrap())
        .with_children(|parent| {
            leaderboard_display_box = Some(
                parent
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Px(window.width()), Val::Px(window.height())),
                            position_type: PositionType::Absolute,
                            position: Rect {
                                left: Val::Px(0.0),
                                top: Val::Px(0.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        material: base_color_materials.colors[0].clone(),
                        ..Default::default()
                    })
                    .insert(UIComponent)
                    .with_children(|parent| {
                        // spawn border
                        let mut spawn_color = |y: usize, x: usize| {
                            parent
                                .spawn_bundle(NodeBundle {
                                    style: Style {
                                        size: Size::new(
                                            Val::Px(PIXEL_SCALE as f32),
                                            Val::Px(PIXEL_SCALE as f32),
                                        ),
                                        position_type: PositionType::Absolute,
                                        position: Rect {
                                            left: Val::Px((x * PIXEL_SCALE) as f32),
                                            top: Val::Px((y * PIXEL_SCALE) as f32),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    material: base_color_materials.colors[rand::thread_rng()
                                        .gen_range(0..base_color_materials.colors.len())]
                                    .clone(),
                                    ..Default::default()
                                })
                                .insert(UIComponent);
                        };

                        let height = window.height() as usize / PIXEL_SCALE;
                        let width = window.width() as usize / PIXEL_SCALE;
                        for y in 0..height {
                            spawn_color(y, 0);
                            spawn_color(y, 1);
                            spawn_color(y, width - 2);
                            spawn_color(y, width - 1);
                        }
                        for x in 2..width - 2 {
                            spawn_color(0, x);
                            spawn_color(1, x);
                            spawn_color(height - 2, x);
                            spawn_color(height - 1, x);
                        }

                        for (penguin, score) in &leaderboard.scores {
                            // spawn penguin portrait
                            parent
                                .spawn_bundle(NodeBundle {
                                    style: Style {
                                        size: Size::new(
                                            Val::Px(TILE_WIDTH as f32),
                                            Val::Px(TILE_HEIGHT as f32),
                                        ),
                                        position_type: PositionType::Absolute,
                                        position: Rect {
                                            left: Val::Px(4.0 * PIXEL_SCALE as f32),
                                            top: Val::Px(
                                                ((6 + penguin.0 * 12) * PIXEL_SCALE) as f32,
                                            ),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    material: base_color_materials.colors[2].clone(),
                                    ..Default::default()
                                })
                                .insert(UIComponent)
                                .with_children(|parent| {
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
                                                .get_penguin_texture(*penguin)
                                                .clone(),
                                            ..Default::default()
                                        })
                                        .insert(UIComponent);
                                });

                            // spawn penguin trophies
                            for i in 0..*score {
                                parent
                                    .spawn_bundle(ImageBundle {
                                        style: Style {
                                            size: Size::new(
                                                Val::Px(5.0 * PIXEL_SCALE as f32),
                                                Val::Px(7.0 * PIXEL_SCALE as f32),
                                            ),
                                            position_type: PositionType::Absolute,
                                            position: Rect {
                                                top: Val::Px(
                                                    ((7 + penguin.0 * 12) * PIXEL_SCALE) as f32,
                                                ),
                                                left: Val::Px(((15 + i * 9) * PIXEL_SCALE) as f32),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        material: textures.trophy.clone(),
                                        ..Default::default()
                                    })
                                    .insert(UIComponent);
                            }

                            if let Some(round_winner_penguin) = leaderboard.last_round_winner {
                                if *penguin == round_winner_penguin {
                                    let mut place_text = |y, x, str: &str, c: usize| {
                                        parent
                                            .spawn_bundle(TextBundle {
                                                text: Text::with_section(
                                                    str.to_string(),
                                                    TextStyle {
                                                        font: fonts.mono.clone(),
                                                        font_size: 2.0 * PIXEL_SCALE as f32,
                                                        color: COLORS[c].into(),
                                                    },
                                                    TextAlignment::default(),
                                                ),
                                                style: Style {
                                                    position_type: PositionType::Absolute,
                                                    position: Rect {
                                                        top: Val::Px(y as f32 * PIXEL_SCALE as f32),
                                                        left: Val::Px(
                                                            x as f32 * PIXEL_SCALE as f32,
                                                        ),
                                                        ..Default::default()
                                                    },
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            })
                                            .insert(UIComponent);
                                    };

                                    place_text(
                                        6 + penguin.0 * 12,
                                        15 + (*score - 1) * 9 - 2,
                                        "*",
                                        15,
                                    );
                                    place_text(
                                        8 + penguin.0 * 12,
                                        15 + (*score - 1) * 9 + 6,
                                        "*",
                                        15,
                                    );
                                    place_text(
                                        10 + penguin.0 * 12,
                                        15 + (*score - 1) * 9 - 1,
                                        "*",
                                        15,
                                    );
                                }
                            }
                        }
                    })
                    .id(),
            );
        });

    commands.insert_resource(LeaderboardDisplayContext {
        leaderboard_display_box: leaderboard_display_box.unwrap(),
        timer: Timer::from_seconds(1.5, false),
    });
}

pub fn leaderboard_display_update(
    mut commands: Commands,
    textures: Res<Textures>,
    hud_materials: Res<HUDMaterials>,
    time: Res<Time>,
    mut leaderboard_display_context: ResMut<LeaderboardDisplayContext>,
    leaderboard: Res<Leaderboard>,
    map_size: Res<MapSize>,
    battle_mode_configuration: Res<BattleModeConfiguration>,
    game_option_store: Res<GameOptionStore>,
    mut game_timer: ResMut<GameTimer>,
    mut wall_of_death: ResMut<WallOfDeath>,
    mut state: ResMut<State<AppState>>,
    query: Query<Entity, With<PenguinPortraitDisplay>>,
    mut query2: Query<&mut Text, With<GameTimerDisplay>>,
) {
    leaderboard_display_context.timer.tick(time.delta());
    if leaderboard_display_context.timer.finished() {
        commands
            .entity(leaderboard_display_context.leaderboard_display_box)
            .despawn_recursive();
        commands.remove_resource::<LeaderboardDisplayContext>();

        if let Some((penguin, _)) = leaderboard
            .scores
            .iter()
            .find(|(_, s)| **s == leaderboard.winning_score)
        {
            println!("Tournament complete! Winner: {:?}", penguin.0);
            commands.remove_resource::<BattleModeConfiguration>();
            state.replace(AppState::MainMenu).unwrap();
        } else {
            // spawn players & map again
            let amount_of_players = battle_mode_configuration.amount_of_players;
            let amount_of_bots = battle_mode_configuration.amount_of_bots;
            // TODO: put in setup and use as resource instead of recalculating?
            let (_, percent_of_passable_positions_to_fill) =
                get_battle_mode_map_size_fill(amount_of_players + amount_of_bots);
            let (player_spawn_positions, penguin_tags) = spawn_battle_mode_players(
                &mut commands,
                &textures,
                *map_size,
                amount_of_players,
                amount_of_bots,
            );

            commands
                .entity(query.single().unwrap())
                .with_children(|parent| {
                    init_penguin_portraits(parent, &penguin_tags, &hud_materials, &textures);
                });

            let wall_entity_reveal_groups = spawn_map(
                &mut commands,
                &textures,
                *map_size,
                percent_of_passable_positions_to_fill,
                true,
                &player_spawn_positions,
                &[],
                false,
            );

            game_timer.0.reset();
            // update HUD clock so we don't get last round time during the map transition
            let remaining_seconds = game_timer.0.duration().as_secs_f32() as usize;
            query2.single_mut().unwrap().sections[0].value = format_hud_time(remaining_seconds);

            *wall_of_death = WallOfDeath::Dormant(Timer::from_seconds(
                game_timer.0.duration().as_secs_f32() / 2.0,
                false,
            ));

            if game_option_store.get(GameOption::Transition) {
                commands.insert_resource(MapTransitionInput {
                    wall_entity_reveal_groups,
                });
                state.set(AppState::MapTransition).unwrap();
            } else {
                state.pop().unwrap();
            }
        }
    }
}

pub fn setup_secret_mode(
    mut commands: Commands,
    audio: Res<Audio>,
    sounds: Res<Sounds>,
    mut textures: ResMut<Textures>,
    base_color_materials: Res<BaseColorMaterials>,
    hud_materials: Res<HUDMaterials>,
    fonts: Res<Fonts>,
    game_option_store: Res<GameOptionStore>,
    mut state: ResMut<State<AppState>>,
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

    // spawn camera
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
                &textures,
                world_id,
                &[],
                *state.current(),
                None,
                None,
                None,
            );
        });

    let wall_entity_reveal_groups = spawn_map(
        &mut commands,
        &textures,
        map_size,
        0.0,
        false,
        &[player_spawn_position],
        &[],
        false,
    );

    commands.insert_resource(world_id);
    commands.insert_resource(map_size);
    commands.insert_resource(SecretLevelContext {
        state: SecretLevelState::Initial(Timer::from_seconds(2.5, false)),
        pattern: PATTERN,
    });

    if game_option_store.get(GameOption::Transition) {
        commands.insert_resource(MapTransitionInput {
            wall_entity_reveal_groups,
        });
        state.push(AppState::MapTransition).unwrap();
    }
}

pub fn update_secret_mode(
    mut commands: Commands,
    time: Res<Time>,
    textures: Res<Textures>,
    fonts: Res<Fonts>,
    map_size: Res<MapSize>,
    world_id: Res<WorldID>,
    mut context: ResMut<SecretLevelContext>,
    mut state: ResMut<State<AppState>>,
    mut q: QuerySet<(
        QueryState<(Entity, &mut Position, &mut Transform), With<Bomb>>,
        QueryState<&Position, With<Wall>>,
    )>,
    mut query: Query<(Entity, &mut Handle<ColorMaterial>, &mut BaseMaterial), With<Player>>,
) {
    let pattern = context.pattern;

    loop {
        let new_state = match &mut context.state {
            SecretLevelState::Initial(timer) => {
                timer.tick(time.delta());
                if timer.finished() {
                    Some(SecretLevelState::Started {
                        move_cooldown: Cooldown::from_seconds(0.25),
                        round_progress: 0,
                        round: 0,
                    })
                } else {
                    None
                }
            }
            SecretLevelState::Started {
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
            SecretLevelState::Stopping(timer) => {
                timer.tick(time.delta());

                if timer.just_finished() {
                    state.overwrite_pop().unwrap();
                }

                None
            }
        };

        if let Some(new_state) = new_state {
            context.state = new_state;
        } else {
            break;
        }
    }
}

pub fn finish_secret_mode(
    mut commands: Commands,
    mut context: ResMut<SecretLevelContext>,
    query: Query<(Entity, &Position), With<Player>>,
    query2: Query<(Entity, &Bomb, &Position)>,
    mut ev_explosion: EventWriter<ExplosionEvent>,
) {
    let (player_entity, player_position) = query.single().unwrap();
    if query2.iter().any(|(_, _, p)| *p == *player_position) {
        context.state = SecretLevelState::Stopping(Timer::from_seconds(0.5, false));

        commands.entity(player_entity).remove::<HumanControlled>();
        for (entity, bomb, position) in query2.iter() {
            commands.entity(entity).despawn_recursive();
            ev_explosion.send(ExplosionEvent(*position, bomb.range));
        }
    }
}

pub fn stop_audio(audio: Res<Audio>) {
    audio.stop();
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
        state.replace(AppState::MainMenu).unwrap();
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
        let name = if name.len() == 0 {
            String::from("<unnamed_player>")
        } else {
            name.clone()
        };

        persistent_high_scores.insert_score(name, game_score.0);
        commands.remove_resource::<HighScoreNameInputContext>();
        state.replace(AppState::MainMenu).unwrap();
        keyboard_input.reset(KeyCode::Return);
    }
}
