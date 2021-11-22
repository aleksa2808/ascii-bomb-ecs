use bevy::prelude::*;

use crate::{
    common::{
        constants::PIXEL_SCALE,
        resources::{Fonts, GameOption, GameOptionStore, HighScores},
    },
    game::types::BotDifficulty,
    main_menu::resources::BattleModeSubMenuStep,
};

use super::{
    components::{BattleModeSubMenuContentBox, Cursor},
    resources::{BattleModeSubMenuState, MenuMaterials, MenuType},
};

pub fn spawn_menu_type(
    parent: &mut ChildBuilder,
    menu_type: &MenuType,
    fonts: &Fonts,
    menu_materials: &MenuMaterials,
    game_option_store: &GameOptionStore,
    high_scores: &HighScores,
) {
    match menu_type {
        MenuType::SelectableItems(selectable_items) => {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    selectable_items.get_item_names().join("\n\n"),
                    TextStyle {
                        font: fonts.mono.clone(),
                        font_size: 2.0 * PIXEL_SCALE as f32,
                        color: menu_materials.modal_foreground_color,
                    },
                    TextAlignment::default(),
                ),
                style: Style {
                    position_type: PositionType::Absolute,
                    position: Rect {
                        top: Val::Px(2.0 * PIXEL_SCALE as f32),
                        left: Val::Px(3.0 * PIXEL_SCALE as f32),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            });

            // spawn cursor
            parent
                .spawn_bundle(TextBundle {
                    text: Text::with_section(
                        "»",
                        TextStyle {
                            font: fonts.mono.clone(),
                            font_size: 2.0 * PIXEL_SCALE as f32,
                            color: menu_materials.modal_foreground_color,
                        },
                        TextAlignment::default(),
                    ),
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: Rect {
                            top: Val::Px(
                                ((2 + selectable_items.get_cursor_position() * 4) * PIXEL_SCALE)
                                    as f32,
                            ),
                            left: Val::Px(PIXEL_SCALE as f32),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Cursor);
        }
        MenuType::ToggleableOptions(toggleable_options) => {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    toggleable_options
                        .get_options()
                        .iter()
                        .map(|o| {
                            format!(
                                "{: <12}-       {}",
                                match o {
                                    GameOption::Sound => "SOUND",
                                    GameOption::Demo => "DEMO",
                                    GameOption::Transition => "TRANSITION",
                                },
                                if game_option_store.get(*o) {
                                    "ON"
                                } else {
                                    "OFF"
                                }
                            )
                        })
                        .collect::<Vec<String>>()
                        .join("\n\n"),
                    TextStyle {
                        font: fonts.mono.clone(),
                        font_size: 2.0 * PIXEL_SCALE as f32,
                        color: menu_materials.modal_foreground_color,
                    },
                    TextAlignment::default(),
                ),
                style: Style {
                    position_type: PositionType::Absolute,
                    position: Rect {
                        top: Val::Px(2.0 * PIXEL_SCALE as f32),
                        left: Val::Px(3.0 * PIXEL_SCALE as f32),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            });

            // spawn cursor
            parent
                .spawn_bundle(TextBundle {
                    text: Text::with_section(
                        "»",
                        TextStyle {
                            font: fonts.mono.clone(),
                            font_size: 2.0 * PIXEL_SCALE as f32,
                            color: menu_materials.modal_foreground_color,
                        },
                        TextAlignment::default(),
                    ),
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: Rect {
                            top: Val::Px(
                                ((2 + toggleable_options.get_cursor_position() * 4) * PIXEL_SCALE)
                                    as f32,
                            ),
                            left: Val::Px(PIXEL_SCALE as f32),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Cursor);
        }
        MenuType::StaticText(static_text) | MenuType::ControlsScreen(static_text) => {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    *static_text,
                    TextStyle {
                        font: fonts.mono.clone(),
                        font_size: 2.0 * PIXEL_SCALE as f32,
                        color: menu_materials.modal_foreground_color,
                    },
                    TextAlignment::default(),
                ),
                style: Style {
                    position_type: PositionType::Absolute,
                    position: Rect {
                        top: Val::Px(2.0 * PIXEL_SCALE as f32),
                        left: Val::Px(PIXEL_SCALE as f32),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            });

            // continue button
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(
                            Val::Px(8.0 * PIXEL_SCALE as f32),
                            Val::Px(2.0 * PIXEL_SCALE as f32),
                        ),
                        position_type: PositionType::Absolute,
                        position: Rect {
                            left: Val::Px(15.0 * PIXEL_SCALE as f32),
                            top: Val::Px(32.0 * PIXEL_SCALE as f32),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    material: menu_materials.modal_foreground.clone(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "CONTINUE",
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
                                top: Val::Px(0.0),
                                left: Val::Px(0.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    });
                });
        }
        MenuType::HallOfFame => {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "HIGH-SCORES",
                    TextStyle {
                        font: fonts.mono.clone(),
                        font_size: 2.0 * PIXEL_SCALE as f32,
                        color: menu_materials.modal_foreground_color,
                    },
                    TextAlignment::default(),
                ),
                style: Style {
                    position_type: PositionType::Absolute,
                    position: Rect {
                        top: Val::Px(2.0 * PIXEL_SCALE as f32),
                        left: Val::Px(13.0 * PIXEL_SCALE as f32),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            });

            let high_scores_text = (0..HighScores::HIGH_SCORES_MAX_SIZE)
                .map(|i| {
                    if let Some(score) = high_scores.0.get(i) {
                        format!("{:>2}. {:<20}{:>10}", i + 1, score.0, score.1)
                    } else {
                        format!("{:>2}. -----                        0", i + 1)
                    }
                })
                .collect::<Vec<String>>()
                .join("\n");
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    high_scores_text,
                    TextStyle {
                        font: fonts.mono.clone(),
                        font_size: 2.0 * PIXEL_SCALE as f32,
                        color: menu_materials.modal_foreground_color,
                    },
                    TextAlignment::default(),
                ),
                style: Style {
                    position_type: PositionType::Absolute,
                    position: Rect {
                        top: Val::Px(6.0 * PIXEL_SCALE as f32),
                        left: Val::Px(PIXEL_SCALE as f32),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            });

            // continue button
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(
                            Val::Px(8.0 * PIXEL_SCALE as f32),
                            Val::Px(2.0 * PIXEL_SCALE as f32),
                        ),
                        position_type: PositionType::Absolute,
                        position: Rect {
                            left: Val::Px(15.0 * PIXEL_SCALE as f32),
                            top: Val::Px(32.0 * PIXEL_SCALE as f32),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    material: menu_materials.modal_foreground.clone(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "CONTINUE",
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
                                top: Val::Px(0.0),
                                left: Val::Px(0.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    });
                });
        }
    }
}

pub fn spawn_battle_mode_sub_menu_modal(
    parent: &mut ChildBuilder,
    sub_menu_state: &BattleModeSubMenuState,
    fonts: &Fonts,
    menu_materials: &MenuMaterials,
) {
    parent
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(
                    Val::Px(26.0 * PIXEL_SCALE as f32),
                    Val::Px(14.0 * PIXEL_SCALE as f32),
                ),
                position_type: PositionType::Absolute,
                position: Rect {
                    left: Val::Px(6.0 * PIXEL_SCALE as f32),
                    top: Val::Px(12.0 * PIXEL_SCALE as f32),
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
        .with_children(|parent| {
            // spawn modal border
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    r#"
┌────────────────────────┐
│                        │
│                        │
│                        │
│                        │
│                        │
└────────────────────────┘
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
            });

            // spawn initial content
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        ..Default::default()
                    },
                    material: menu_materials.modal_background.clone(),
                    ..Default::default()
                })
                .insert(BattleModeSubMenuContentBox)
                .with_children(|parent| {
                    spawn_battle_mode_sub_menu_content(
                        parent,
                        sub_menu_state,
                        fonts,
                        menu_materials,
                    );
                });
        });
}

pub fn spawn_battle_mode_sub_menu_content(
    parent: &mut ChildBuilder,
    sub_menu_state: &BattleModeSubMenuState,
    fonts: &Fonts,
    menu_materials: &MenuMaterials,
) {
    parent.spawn_bundle(TextBundle {
        text: Text::with_section(
            match sub_menu_state.step {
                BattleModeSubMenuStep::AmountOfPlayers => "AMOUNT OF PLAYERS:",
                BattleModeSubMenuStep::AmountOfBots => "AMOUNT OF BOTS:",
                BattleModeSubMenuStep::WinningScore => "AMOUNT OF WINS:",
                BattleModeSubMenuStep::Difficulty => "DIFFICULTY:",
            },
            TextStyle {
                font: fonts.mono.clone(),
                font_size: 2.0 * PIXEL_SCALE as f32,
                color: menu_materials.modal_foreground_color,
            },
            TextAlignment::default(),
        ),
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(2.0 * PIXEL_SCALE as f32),
                left: Val::Px(PIXEL_SCALE as f32),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });

    parent.spawn_bundle(TextBundle {
        text: Text::with_section(
            format!(
                "« {} »",
                match sub_menu_state.step {
                    BattleModeSubMenuStep::AmountOfPlayers =>
                        sub_menu_state.amount_of_players.value().to_string(),
                    BattleModeSubMenuStep::AmountOfBots =>
                        sub_menu_state.amount_of_bots.value().to_string(),
                    BattleModeSubMenuStep::WinningScore =>
                        sub_menu_state.winning_score.value().to_string(),
                    BattleModeSubMenuStep::Difficulty =>
                        String::from(match sub_menu_state.difficulty.value() {
                            BotDifficulty::Easy => "EASY",
                            BotDifficulty::Medium => "MEDIUM",
                            BotDifficulty::Hard => "HARD",
                        }),
                }
            ),
            TextStyle {
                font: fonts.mono.clone(),
                font_size: 2.0 * PIXEL_SCALE as f32,
                color: menu_materials.modal_foreground_color,
            },
            TextAlignment::default(),
        ),
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(4.0 * PIXEL_SCALE as f32),
                left: Val::Px(PIXEL_SCALE as f32),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });
}
