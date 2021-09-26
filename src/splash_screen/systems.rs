use bevy::prelude::*;

use crate::{
    constants::{COLORS, PIXEL_SCALE},
    main_menu::{MENU_HEIGHT, MENU_WIDTH},
    resources::{BaseColorMaterials, Fonts},
    AppState,
};

use super::{
    constants::{SPLASH_SCREEN_TEXT_LEFT, SPLASH_SCREEN_TEXT_RIGHT},
    resources::{SplashScreenContext, SplashScreenTextState},
};

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
                state.replace(AppState::MainMenu).unwrap();
            }
        }
    }
}

pub fn teardown(mut commands: Commands, query: Query<Entity>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    commands.remove_resource::<SplashScreenContext>();
}
