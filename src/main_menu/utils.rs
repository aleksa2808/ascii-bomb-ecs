use bevy::prelude::*;

use crate::{
    common::{
        constants::PIXEL_SCALE,
        resources::{Fonts, GameOption, GameOptionStore, HighScores},
    },
    game::types::BotDifficulty,
};

use super::{
    components::{BattleModeSubMenuContentBox, Cursor},
    resources::{BattleModeSubMenuState, BattleModeSubMenuStep, MenuColors, MenuType},
};

pub fn spawn_menu_type(
    parent: &mut ChildBuilder,
    menu_type: &MenuType,
    fonts: &Fonts,
    menu_colors: &MenuColors,
    game_option_store: &GameOptionStore,
    high_scores: &HighScores,
) {
    match menu_type {
        MenuType::SelectableItems(selectable_items) => {
            parent.spawn(TextBundle {
                text: Text::from_section(
                    selectable_items.get_item_names().join("\n\n"),
                    TextStyle {
                        font: fonts.mono.clone(),
                        font_size: 2.0 * PIXEL_SCALE as f32,
                        color: menu_colors.modal_foreground_color,
                    },
                ),
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Px(2.0 * PIXEL_SCALE as f32),
                        left: Val::Px(3.0 * PIXEL_SCALE as f32),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            });

            // spawn cursor
            parent.spawn((
                TextBundle {
                    text: Text::from_section(
                        "»",
                        TextStyle {
                            font: fonts.mono.clone(),
                            font_size: 2.0 * PIXEL_SCALE as f32,
                            color: menu_colors.modal_foreground_color,
                        },
                    ),
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: UiRect {
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
                },
                Cursor,
            ));
        }
        MenuType::ToggleableOptions(toggleable_options) => {
            parent.spawn(TextBundle {
                text: Text::from_section(
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
                        color: menu_colors.modal_foreground_color,
                    },
                ),
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Px(2.0 * PIXEL_SCALE as f32),
                        left: Val::Px(3.0 * PIXEL_SCALE as f32),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            });

            // spawn cursor
            parent.spawn((
                TextBundle {
                    text: Text::from_section(
                        "»",
                        TextStyle {
                            font: fonts.mono.clone(),
                            font_size: 2.0 * PIXEL_SCALE as f32,
                            color: menu_colors.modal_foreground_color,
                        },
                    ),
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: UiRect {
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
                },
                Cursor,
            ));
        }
        MenuType::StaticText(static_text) | MenuType::ControlsScreen(static_text) => {
            parent.spawn(TextBundle {
                text: Text::from_section(
                    *static_text,
                    TextStyle {
                        font: fonts.mono.clone(),
                        font_size: 2.0 * PIXEL_SCALE as f32,
                        color: menu_colors.modal_foreground_color,
                    },
                ),
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
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
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(
                            Val::Px(8.0 * PIXEL_SCALE as f32),
                            Val::Px(2.0 * PIXEL_SCALE as f32),
                        ),
                        position_type: PositionType::Absolute,
                        position: UiRect {
                            left: Val::Px(15.0 * PIXEL_SCALE as f32),
                            top: Val::Px(32.0 * PIXEL_SCALE as f32),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    background_color: menu_colors.modal_foreground_color.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "CONTINUE",
                            TextStyle {
                                font: fonts.mono.clone(),
                                font_size: 2.0 * PIXEL_SCALE as f32,
                                color: menu_colors.modal_background_color,
                            },
                        ),
                        style: Style {
                            position_type: PositionType::Absolute,
                            position: UiRect {
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
            parent.spawn(TextBundle {
                text: Text::from_section(
                    "HIGH-SCORES",
                    TextStyle {
                        font: fonts.mono.clone(),
                        font_size: 2.0 * PIXEL_SCALE as f32,
                        color: menu_colors.modal_foreground_color,
                    },
                ),
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
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
            parent.spawn(TextBundle {
                text: Text::from_section(
                    high_scores_text,
                    TextStyle {
                        font: fonts.mono.clone(),
                        font_size: 2.0 * PIXEL_SCALE as f32,
                        color: menu_colors.modal_foreground_color,
                    },
                ),
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
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
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(
                            Val::Px(8.0 * PIXEL_SCALE as f32),
                            Val::Px(2.0 * PIXEL_SCALE as f32),
                        ),
                        position_type: PositionType::Absolute,
                        position: UiRect {
                            left: Val::Px(15.0 * PIXEL_SCALE as f32),
                            top: Val::Px(32.0 * PIXEL_SCALE as f32),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    background_color: menu_colors.modal_foreground_color.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "CONTINUE",
                            TextStyle {
                                font: fonts.mono.clone(),
                                font_size: 2.0 * PIXEL_SCALE as f32,
                                color: menu_colors.modal_background_color,
                            },
                        ),
                        style: Style {
                            position_type: PositionType::Absolute,
                            position: UiRect {
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
    menu_colors: &MenuColors,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(
                    Val::Px(26.0 * PIXEL_SCALE as f32),
                    Val::Px(14.0 * PIXEL_SCALE as f32),
                ),
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Px(6.0 * PIXEL_SCALE as f32),
                    top: Val::Px(12.0 * PIXEL_SCALE as f32),
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
            background_color: menu_colors.modal_foreground_color.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            // spawn modal border
            parent.spawn(TextBundle {
                text: Text::from_section(
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

            // spawn initial content
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                            ..Default::default()
                        },
                        background_color: menu_colors.modal_background_color.into(),
                        ..Default::default()
                    },
                    BattleModeSubMenuContentBox,
                ))
                .with_children(|parent| {
                    spawn_battle_mode_sub_menu_content(parent, sub_menu_state, fonts, menu_colors);
                });
        });
}

pub fn spawn_battle_mode_sub_menu_content(
    parent: &mut ChildBuilder,
    sub_menu_state: &BattleModeSubMenuState,
    fonts: &Fonts,
    menu_colors: &MenuColors,
) {
    parent.spawn(TextBundle {
        text: Text::from_section(
            match sub_menu_state.step {
                BattleModeSubMenuStep::AmountOfPlayers => "AMOUNT OF PLAYERS:",
                BattleModeSubMenuStep::AmountOfBots => "AMOUNT OF BOTS:",
                BattleModeSubMenuStep::WinningScore => "AMOUNT OF WINS:",
                BattleModeSubMenuStep::BotDifficulty => "DIFFICULTY:",
            },
            TextStyle {
                font: fonts.mono.clone(),
                font_size: 2.0 * PIXEL_SCALE as f32,
                color: menu_colors.modal_foreground_color,
            },
        ),
        style: Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(2.0 * PIXEL_SCALE as f32),
                left: Val::Px(PIXEL_SCALE as f32),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });

    parent.spawn(TextBundle {
        text: Text::from_section(
            format!(
                "« {} »",
                match sub_menu_state.step {
                    BattleModeSubMenuStep::AmountOfPlayers => sub_menu_state
                        .amount_of_actors
                        .amount_of_players()
                        .to_string(),
                    BattleModeSubMenuStep::AmountOfBots =>
                        sub_menu_state.amount_of_actors.amount_of_bots().to_string(),
                    BattleModeSubMenuStep::WinningScore =>
                        sub_menu_state.winning_score.value().to_string(),
                    BattleModeSubMenuStep::BotDifficulty =>
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
                color: menu_colors.modal_foreground_color,
            },
        ),
        style: Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(4.0 * PIXEL_SCALE as f32),
                left: Val::Px(PIXEL_SCALE as f32),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });
}
