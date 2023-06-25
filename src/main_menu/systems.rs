use bevy::{app::AppExit, prelude::*, utils::HashMap};

use crate::{
    audio::Audio,
    battle_mode::BattleModeConfiguration,
    common::{
        constants::{COLORS, PIXEL_SCALE},
        resources::{
            Fonts, GameOption, GameOptionStore, InputActionStatusTracker, PersistentHighScores,
        },
        types::InputAction,
    },
    game::types::BotDifficulty,
    AppState,
};

use super::{
    components::{BattleModeSubMenuContentBox, MenuContentBox},
    constants::{DEMO_MODE_START_TIMER_DURATION_SECS, MENU_HEIGHT, MENU_WIDTH},
    resources::{
        BattleModeSubMenuState, BattleModeSubMenuStep, DemoModeStartTimer, MainMenuSoundEffects,
        MenuAction, MenuBackgroundAnimationContext, MenuBackgroundEntityChangeParameters,
        MenuBackgroundEntityValues, MenuColors, MenuState, MenuType,
    },
    utils::{
        spawn_battle_mode_sub_menu_content, spawn_battle_mode_sub_menu_modal, spawn_menu_type,
    },
};

pub fn resize_window(mut windows: ResMut<Windows>) {
    windows
        .get_primary_mut()
        .unwrap()
        .set_resolution(MENU_WIDTH as f32, MENU_HEIGHT as f32);
}

pub fn setup_menu(
    fonts: Res<Fonts>,
    mut commands: Commands,
    menu_colors: Res<MenuColors>,
    menu_state: Res<MenuState>,
    game_option_store: Res<GameOptionStore>,
    persistent_high_scores: Res<PersistentHighScores>,
) {
    commands.spawn_bundle(Camera2dBundle::default());

    let mut menu_background_animation_context = None;
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                ..Default::default()
            },
            color: menu_colors.background_color.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            let title_text = r#"
 ____   ____  __  __ ____  ______ _____  __  __          _   _ 
|  _ \ / __ \|  \/  |  _ \|  ____|  __ \|  \/  |   /\   | \ | |
| |_) | |  | | \  / | |_) | |__  | |__) | \  / |  /  \  |  \| |
|  _ <| |  | | |\/| |  _ <|  __| |  _  /| |\/| | / /\ \ | . ` |
| |_) | |__| | |  | | |_) | |____| | \ \| |  | |/ ____ \| |\  |
|____/ \____/|_|  |_|____/|______|_|  \_\_|  |_/_/    \_\_| \_|
"#;
            parent.spawn_bundle(TextBundle {
                text: Text::from_section(
                    title_text.to_string(),
                    TextStyle {
                        font: fonts.mono.clone(),
                        font_size: 2.0 * PIXEL_SCALE as f32,
                        color: COLORS[15].into(),
                    },
                ),
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Px(12.0 * PIXEL_SCALE as f32),
                        left: Val::Px(17.0 * PIXEL_SCALE as f32),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            });

            let mut place_text = |y, x, str: &str, c: usize| {
                parent
                    .spawn_bundle(TextBundle {
                        text: Text::from_section(
                            str.to_string(),
                            TextStyle {
                                font: fonts.mono.clone(),
                                font_size: 2.0 * PIXEL_SCALE as f32,
                                color: COLORS[c].into(),
                            },
                        ),
                        style: Style {
                            position_type: PositionType::Absolute,
                            position: UiRect {
                                top: Val::Px(y as f32 * 2.0 * PIXEL_SCALE as f32),
                                left: Val::Px(x as f32 * PIXEL_SCALE as f32),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    })
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

            let mut entity_change_parameters = HashMap::default();

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
                        position: UiRect {
                            left: Val::Px(30.0 * PIXEL_SCALE as f32),
                            top: Val::Px(36.0 * PIXEL_SCALE as f32),
                            ..Default::default()
                        },
                        border: UiRect {
                            left: Val::Px(PIXEL_SCALE as f32),
                            top: Val::Px(2.0 * PIXEL_SCALE as f32),
                            right: Val::Px(PIXEL_SCALE as f32),
                            bottom: Val::Px(2.0 * PIXEL_SCALE as f32),
                        },
                        ..Default::default()
                    },
                    color: menu_colors.modal_foreground_color.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    // spawn modal border
                    parent.spawn_bundle(TextBundle {
                        text: Text::from_section(
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
                                color: menu_colors.modal_background_color,
                            },
                        ),
                        style: Style {
                            position_type: PositionType::Absolute,
                            position: UiRect {
                                top: Val::Px(-2.0 * PIXEL_SCALE as f32),
                                left: Val::Px(-1.0 * PIXEL_SCALE as f32),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    });

                    // spawn menu type
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                ..Default::default()
                            },
                            color: menu_colors.modal_background_color.into(),
                            ..Default::default()
                        })
                        .insert(MenuContentBox)
                        .with_children(|parent| {
                            spawn_menu_type(
                                parent,
                                menu_state.get_current_menu(),
                                &fonts,
                                &menu_colors,
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
    sounds: Res<MainMenuSoundEffects>,
    fonts: Res<Fonts>,
    menu_colors: Res<MenuColors>,
    mut state: ResMut<State<AppState>>,
    mut menu_state: ResMut<MenuState>,
    mut game_option_store: ResMut<GameOptionStore>,
    persistent_high_scores: Res<PersistentHighScores>,
    mut inputs: ResMut<InputActionStatusTracker>,
    mut query: Query<(Entity, &Children), With<MenuContentBox>>,
    mut query3: Query<(Entity, &Children), With<BattleModeSubMenuContentBox>>,
    mut ev_exit: EventWriter<AppExit>,
) {
    let mut menu_changed = false;
    if let Some(ref mut sub_menu_state) = menu_state.battle_mode_sub_menu_state {
        if inputs.is_active(InputAction::Left) {
            match sub_menu_state.step {
                BattleModeSubMenuStep::AmountOfPlayers => sub_menu_state
                    .amount_of_actors
                    .decrement_amount_of_players(),
                BattleModeSubMenuStep::AmountOfBots => {
                    sub_menu_state.amount_of_actors.decrement_amount_of_bots()
                }
                BattleModeSubMenuStep::WinningScore => sub_menu_state.winning_score.decrement(),
                BattleModeSubMenuStep::BotDifficulty => sub_menu_state.difficulty.decrement(),
            }
            menu_changed = true;
        }

        if inputs.is_active(InputAction::Right) {
            match sub_menu_state.step {
                BattleModeSubMenuStep::AmountOfPlayers => sub_menu_state
                    .amount_of_actors
                    .increment_amount_of_players(),
                BattleModeSubMenuStep::AmountOfBots => {
                    sub_menu_state.amount_of_actors.increment_amount_of_bots()
                }
                BattleModeSubMenuStep::WinningScore => sub_menu_state.winning_score.increment(),
                BattleModeSubMenuStep::BotDifficulty => sub_menu_state.difficulty.increment(),
            }
            menu_changed = true;
        }

        if inputs.is_active(InputAction::Return) || inputs.is_active(InputAction::Space) {
            match sub_menu_state.step {
                BattleModeSubMenuStep::AmountOfPlayers => {
                    sub_menu_state.step = BattleModeSubMenuStep::AmountOfBots
                }
                BattleModeSubMenuStep::AmountOfBots => {
                    sub_menu_state.step = BattleModeSubMenuStep::WinningScore
                }
                BattleModeSubMenuStep::WinningScore => {
                    sub_menu_state.step = BattleModeSubMenuStep::BotDifficulty
                }
                BattleModeSubMenuStep::BotDifficulty => {
                    commands.insert_resource(BattleModeConfiguration {
                        amount_of_players: sub_menu_state.amount_of_actors.amount_of_players(),
                        amount_of_bots: sub_menu_state.amount_of_actors.amount_of_bots(),
                        winning_score: *sub_menu_state.winning_score.value(),
                        bot_difficulty: *sub_menu_state.difficulty.value(),
                    });

                    menu_state.battle_mode_sub_menu_state = None;
                    state.replace(AppState::BattleMode).unwrap();
                    inputs.clear();
                    return;
                }
            }
            menu_changed = true;
        }

        if inputs.is_active(InputAction::Escape) {
            menu_state.battle_mode_sub_menu_state = None;
            menu_changed = true;
        }
    } else {
        if inputs.is_active(InputAction::Return) || inputs.is_active(InputAction::Space) {
            audio.play(sounds.confirm);
            match menu_state.get_enter_action() {
                MenuAction::SwitchMenu(menu_id) => {
                    menu_state.switch_menu(menu_id);
                    menu_changed = true;
                }
                MenuAction::LaunchStoryMode => {
                    state.replace(AppState::StoryMode).unwrap();
                    inputs.clear();
                    return;
                }
                MenuAction::OpenBattleModeSubMenu => {
                    let sub_menu_state = BattleModeSubMenuState::default();
                    commands.entity(query.single().0).with_children(|parent| {
                        spawn_battle_mode_sub_menu_modal(
                            parent,
                            &sub_menu_state,
                            &fonts,
                            &menu_colors,
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

        if inputs.is_active(InputAction::Escape) && menu_state.back().is_ok() {
            menu_changed = true;
        }

        if inputs.is_active(InputAction::Down) {
            match menu_state.get_current_menu_mut() {
                MenuType::SelectableItems(selectable_items) => {
                    audio.play(sounds.select);
                    selectable_items.cycle_cursor_up();
                    menu_changed = true;
                }
                MenuType::ToggleableOptions(toggleable_options) => {
                    audio.play(sounds.select);
                    toggleable_options.cycle_cursor_up();
                    menu_changed = true;
                }
                MenuType::StaticText(_) | MenuType::ControlsScreen(_) | MenuType::HallOfFame => (),
            }
        }

        if inputs.is_active(InputAction::Up) {
            match menu_state.get_current_menu_mut() {
                MenuType::SelectableItems(selectable_items) => {
                    audio.play(sounds.select);
                    selectable_items.cycle_cursor_down();
                    menu_changed = true;
                }
                MenuType::ToggleableOptions(toggleable_options) => {
                    audio.play(sounds.select);
                    toggleable_options.cycle_cursor_down();
                    menu_changed = true;
                }
                MenuType::StaticText(_) | MenuType::ControlsScreen(_) | MenuType::HallOfFame => (),
            }
        }

        if let MenuType::ControlsScreen(_) = menu_state.get_current_menu() {
            if inputs.is_active(InputAction::F) {
                state.replace(AppState::SecretMode).unwrap();
                inputs.clear();
                return;
            }
        }
    }

    if menu_changed {
        if let Some(sub_menu_state) = &menu_state.battle_mode_sub_menu_state {
            // refresh sub menu
            let (entity, children) = query3.single_mut();
            for child in children.iter() {
                commands.entity(*child).despawn_recursive();
            }
            commands.entity(entity).with_children(|parent| {
                spawn_battle_mode_sub_menu_content(parent, sub_menu_state, &fonts, &menu_colors);
            });
        } else {
            // refresh main menu
            let (entity, children) = query.single_mut();
            for child in children.iter() {
                commands.entity(*child).despawn_recursive();
            }
            commands.entity(entity).with_children(|parent| {
                spawn_menu_type(
                    parent,
                    menu_state.get_current_menu(),
                    &fonts,
                    &menu_colors,
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
    inputs: Res<InputActionStatusTracker>,
    mut state: ResMut<State<AppState>>,
) {
    if let Some(mut demo_mode_start_timer) = demo_mode_start_timer {
        if !inputs.get_active().is_empty() {
            demo_mode_start_timer.0.reset();
        } else {
            demo_mode_start_timer.0.tick(time.delta());
            if demo_mode_start_timer.0.finished() {
                // state switching should fail here if there's a manually triggered state already queued
                if state.replace(AppState::BattleMode).is_ok() {
                    println!("Starting demo mode!");

                    commands.insert_resource(BattleModeConfiguration {
                        amount_of_players: 0,
                        amount_of_bots: 8,
                        winning_score: 1,
                        bot_difficulty: BotDifficulty::Medium,
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

pub fn teardown(mut commands: Commands, query: Query<Entity>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    commands.remove_resource::<DemoModeStartTimer>();
    commands.remove_resource::<MenuBackgroundAnimationContext>();
}
