use bevy::{prelude::*, text::LineHeight};

use crate::{
    common::{
        constants::{COLORS, PIXEL_SCALE},
        resources::Fonts,
    },
    main_menu::{MENU_HEIGHT, MENU_WIDTH},
    AppState,
};

use super::{
    constants::{SPLASH_SCREEN_TEXT_LEFT, SPLASH_SCREEN_TEXT_RIGHT},
    resources::{SplashScreenContext, SplashScreenTextState},
};

pub fn setup_splash_screen(mut commands: Commands, fonts: Res<Fonts>) {
    commands.spawn((Camera2d, DespawnOnExit(AppState::SplashScreen)));

    let right_position = MENU_WIDTH - SPLASH_SCREEN_TEXT_RIGHT.len() * PIXEL_SCALE;

    let mut left_text = None;
    let mut right_text = None;
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..Default::default()
            },
            BackgroundColor(COLORS[0].into()),
            DespawnOnExit(AppState::SplashScreen),
        ))
        .with_children(|parent| {
            left_text = Some(
                parent
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px((MENU_HEIGHT / 2) as f32),
                            left: Val::Px(0.0),
                            ..Default::default()
                        },
                        Text::new(SPLASH_SCREEN_TEXT_LEFT),
                        TextFont {
                            font: fonts.mono.clone(),
                            font_size: 2.0 * PIXEL_SCALE as f32,
                            line_height: LineHeight::RelativeToFont(1.0),
                            ..Default::default()
                        },
                        TextColor(COLORS[15].into()),
                    ))
                    .id(),
            );

            right_text = Some(
                parent
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px((MENU_HEIGHT / 2 + 2 * PIXEL_SCALE) as f32),
                            left: Val::Px(right_position as f32),
                            ..Default::default()
                        },
                        Text::new(SPLASH_SCREEN_TEXT_RIGHT),
                        TextFont {
                            font: fonts.mono.clone(),
                            font_size: 2.0 * PIXEL_SCALE as f32,
                            line_height: LineHeight::RelativeToFont(1.0),
                            ..Default::default()
                        },
                        TextColor(COLORS[15].into()),
                    ))
                    .id(),
            );
        });

    commands.insert_resource(SplashScreenContext {
        left_text: left_text.unwrap(),
        right_text: right_text.unwrap(),
        left_position: 0,
        right_position,
        text_state: SplashScreenTextState::Moving(Timer::from_seconds(0.03, TimerMode::Once)),
    })
}

pub fn splash_screen_update(
    time: Res<Time>,
    mut splash_screen_context: ResMut<SplashScreenContext>,
    mut query: Query<&mut Node>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    match splash_screen_context.text_state {
        SplashScreenTextState::Moving(ref mut timer) => {
            const LEFT_END_POSITION: usize = MENU_WIDTH / 2 - 5 * PIXEL_SCALE;
            const RIGHT_END_POSITION: usize =
                MENU_WIDTH / 2 - (SPLASH_SCREEN_TEXT_RIGHT.len() - 5) * PIXEL_SCALE;

            timer.tick(time.delta());
            if timer.is_finished() {
                timer.reset();

                splash_screen_context.left_position =
                    (splash_screen_context.left_position + PIXEL_SCALE).min(LEFT_END_POSITION);
                query.get_mut(splash_screen_context.left_text).unwrap().left =
                    Val::Px(splash_screen_context.left_position as f32);

                splash_screen_context.right_position = splash_screen_context
                    .right_position
                    .saturating_sub(PIXEL_SCALE)
                    .max(RIGHT_END_POSITION);
                query
                    .get_mut(splash_screen_context.right_text)
                    .unwrap()
                    .left = Val::Px(splash_screen_context.right_position as f32);

                if splash_screen_context.left_position == LEFT_END_POSITION
                    && splash_screen_context.right_position == RIGHT_END_POSITION
                {
                    splash_screen_context.text_state =
                        SplashScreenTextState::Holding(Timer::from_seconds(0.7, TimerMode::Once));
                }
            }
        }
        SplashScreenTextState::Holding(ref mut timer) => {
            timer.tick(time.delta());
            if timer.is_finished() {
                next_state.set(AppState::MainMenu);
            }
        }
    }
}

pub fn teardown(mut commands: Commands) {
    commands.remove_resource::<SplashScreenContext>();
}
