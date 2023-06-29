use bevy::{
    ecs as bevy_ecs,
    prelude::{Entity, Resource},
    time::Timer,
};

pub enum SplashScreenTextState {
    Moving(Timer),
    Holding(Timer),
}

#[derive(Resource)]
pub struct SplashScreenContext {
    pub left_text: Entity,
    pub right_text: Entity,
    pub left_position: usize,
    pub right_position: usize,
    pub text_state: SplashScreenTextState,
}
