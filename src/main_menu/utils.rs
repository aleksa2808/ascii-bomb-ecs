use bevy::{prelude::*, text::LineHeight};

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
    parent: &mut ChildSpawnerCommands,
    menu_type: &MenuType,
    fonts: &Fonts,
    menu_colors: &MenuColors,
    game_option_store: &GameOptionStore,
    high_scores: &HighScores,
) {
    match menu_type {
        MenuType::SelectableItems(selectable_items) => {
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(2.0 * PIXEL_SCALE as f32),
                    left: Val::Px(3.0 * PIXEL_SCALE as f32),
                    ..Default::default()
                },
                Text::new(selectable_items.get_item_names().join("\n\n")),
                TextFont {
                    font: fonts.mono.clone(),
                    font_size: 2.0 * PIXEL_SCALE as f32,
                    line_height: LineHeight::RelativeToFont(1.0),
                    ..Default::default()
                },
                TextColor(menu_colors.modal_foreground_color),
            ));

            // spawn cursor
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(
                        ((2 + selectable_items.get_cursor_position() * 4) * PIXEL_SCALE) as f32,
                    ),
                    left: Val::Px(PIXEL_SCALE as f32),
                    ..Default::default()
                },
                Text::new("»"),
                TextFont {
                    font: fonts.mono.clone(),
                    font_size: 2.0 * PIXEL_SCALE as f32,
                    line_height: LineHeight::RelativeToFont(1.0),
                    ..Default::default()
                },
                TextColor(menu_colors.modal_foreground_color),
                Cursor,
            ));
        }
        MenuType::ToggleableOptions(toggleable_options) => {
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(2.0 * PIXEL_SCALE as f32),
                    left: Val::Px(3.0 * PIXEL_SCALE as f32),
                    ..Default::default()
                },
                Text::new(
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
                ),
                TextFont {
                    font: fonts.mono.clone(),
                    font_size: 2.0 * PIXEL_SCALE as f32,
                    line_height: LineHeight::RelativeToFont(1.0),
                    ..Default::default()
                },
                TextColor(menu_colors.modal_foreground_color),
            ));

            // spawn cursor
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(
                        ((2 + toggleable_options.get_cursor_position() * 4) * PIXEL_SCALE) as f32,
                    ),
                    left: Val::Px(PIXEL_SCALE as f32),
                    ..Default::default()
                },
                Text::new("»"),
                TextFont {
                    font: fonts.mono.clone(),
                    font_size: 2.0 * PIXEL_SCALE as f32,
                    line_height: LineHeight::RelativeToFont(1.0),
                    ..Default::default()
                },
                TextColor(menu_colors.modal_foreground_color),
                Cursor,
            ));
        }
        MenuType::StaticText(static_text) | MenuType::ControlsScreen(static_text) => {
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(2.0 * PIXEL_SCALE as f32),
                    left: Val::Px(PIXEL_SCALE as f32),
                    ..Default::default()
                },
                Text::new(*static_text),
                TextFont {
                    font: fonts.mono.clone(),
                    font_size: 2.0 * PIXEL_SCALE as f32,
                    line_height: LineHeight::RelativeToFont(1.0),
                    ..Default::default()
                },
                TextColor(menu_colors.modal_foreground_color),
            ));

            // continue button
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(15.0 * PIXEL_SCALE as f32),
                        top: Val::Px(32.0 * PIXEL_SCALE as f32),
                        width: Val::Px(8.0 * PIXEL_SCALE as f32),
                        height: Val::Px(2.0 * PIXEL_SCALE as f32),
                        ..Default::default()
                    },
                    BackgroundColor(menu_colors.modal_foreground_color),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(0.0),
                            left: Val::Px(0.0),
                            ..Default::default()
                        },
                        Text::new("CONTINUE"),
                        TextFont {
                            font: fonts.mono.clone(),
                            font_size: 2.0 * PIXEL_SCALE as f32,
                            line_height: LineHeight::RelativeToFont(1.0),
                            ..Default::default()
                        },
                        TextColor(menu_colors.modal_background_color),
                    ));
                });
        }
        MenuType::HallOfFame => {
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(2.0 * PIXEL_SCALE as f32),
                    left: Val::Px(13.0 * PIXEL_SCALE as f32),
                    ..Default::default()
                },
                Text::new("HIGH-SCORES"),
                TextFont {
                    font: fonts.mono.clone(),
                    font_size: 2.0 * PIXEL_SCALE as f32,
                    line_height: LineHeight::RelativeToFont(1.0),
                    ..Default::default()
                },
                TextColor(menu_colors.modal_foreground_color),
            ));

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
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(6.0 * PIXEL_SCALE as f32),
                    left: Val::Px(PIXEL_SCALE as f32),
                    ..Default::default()
                },
                Text::new(high_scores_text),
                TextFont {
                    font: fonts.mono.clone(),
                    font_size: 2.0 * PIXEL_SCALE as f32,
                    line_height: LineHeight::RelativeToFont(1.0),
                    ..Default::default()
                },
                TextColor(menu_colors.modal_foreground_color),
            ));

            // continue button
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(15.0 * PIXEL_SCALE as f32),
                        top: Val::Px(32.0 * PIXEL_SCALE as f32),
                        width: Val::Px(8.0 * PIXEL_SCALE as f32),
                        height: Val::Px(2.0 * PIXEL_SCALE as f32),
                        ..Default::default()
                    },
                    BackgroundColor(menu_colors.modal_foreground_color),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(0.0),
                            left: Val::Px(0.0),
                            ..Default::default()
                        },
                        Text::new("CONTINUE"),
                        TextFont {
                            font: fonts.mono.clone(),
                            font_size: 2.0 * PIXEL_SCALE as f32,
                            line_height: LineHeight::RelativeToFont(1.0),
                            ..Default::default()
                        },
                        TextColor(menu_colors.modal_background_color),
                    ));
                });
        }
    }
}

pub fn spawn_battle_mode_sub_menu_modal(
    parent: &mut ChildSpawnerCommands,
    sub_menu_state: &BattleModeSubMenuState,
    fonts: &Fonts,
    menu_colors: &MenuColors,
) {
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(6.0 * PIXEL_SCALE as f32),
                top: Val::Px(12.0 * PIXEL_SCALE as f32),
                width: Val::Px(26.0 * PIXEL_SCALE as f32),
                height: Val::Px(14.0 * PIXEL_SCALE as f32),
                border: UiRect {
                    left: Val::Px(PIXEL_SCALE as f32),
                    top: Val::Px(2.0 * PIXEL_SCALE as f32),
                    right: Val::Px(PIXEL_SCALE as f32),
                    bottom: Val::Px(2.0 * PIXEL_SCALE as f32),
                },
                ..Default::default()
            },
            BorderColor(menu_colors.modal_foreground_color),
        ))
        .with_children(|parent| {
            // spawn modal border
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(-2.0 * PIXEL_SCALE as f32),
                    left: Val::Px(-1.0 * PIXEL_SCALE as f32),
                    ..Default::default()
                },
                Text::new(
                    r"
┌────────────────────────┐
│                        │
│                        │
│                        │
│                        │
│                        │
└────────────────────────┘
"
                    .trim_matches('\n'),
                ),
                TextFont {
                    font: fonts.mono.clone(),
                    font_size: 2.0 * PIXEL_SCALE as f32,
                    line_height: LineHeight::RelativeToFont(1.0),
                    ..Default::default()
                },
                TextColor(menu_colors.modal_background_color),
            ));

            // spawn initial content
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..Default::default()
                    },
                    BackgroundColor(menu_colors.modal_background_color),
                    BattleModeSubMenuContentBox,
                ))
                .with_children(|parent| {
                    spawn_battle_mode_sub_menu_content(parent, sub_menu_state, fonts, menu_colors);
                });
        });
}

pub fn spawn_battle_mode_sub_menu_content(
    parent: &mut ChildSpawnerCommands,
    sub_menu_state: &BattleModeSubMenuState,
    fonts: &Fonts,
    menu_colors: &MenuColors,
) {
    parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(2.0 * PIXEL_SCALE as f32),
            left: Val::Px(PIXEL_SCALE as f32),
            ..Default::default()
        },
        Text::new(match sub_menu_state.step {
            BattleModeSubMenuStep::AmountOfPlayers => "AMOUNT OF PLAYERS:",
            BattleModeSubMenuStep::AmountOfBots => "AMOUNT OF BOTS:",
            BattleModeSubMenuStep::WinningScore => "AMOUNT OF WINS:",
            BattleModeSubMenuStep::BotDifficulty => "DIFFICULTY:",
        }),
        TextFont {
            font: fonts.mono.clone(),
            font_size: 2.0 * PIXEL_SCALE as f32,
            line_height: LineHeight::RelativeToFont(1.0),
            ..Default::default()
        },
        TextColor(menu_colors.modal_foreground_color),
    ));

    parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(4.0 * PIXEL_SCALE as f32),
            left: Val::Px(PIXEL_SCALE as f32),
            ..Default::default()
        },
        Text::new(format!(
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
        )),
        TextFont {
            font: fonts.mono.clone(),
            font_size: 2.0 * PIXEL_SCALE as f32,
            line_height: LineHeight::RelativeToFont(1.0),
            ..Default::default()
        },
        TextColor(menu_colors.modal_foreground_color),
    ));
}
