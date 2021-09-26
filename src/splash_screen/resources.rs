use bevy::{core::Timer, prelude::Entity};

pub enum SplashScreenTextState {
    Moving(Timer),
    Holding(Timer),
}

pub struct SplashScreenContext {
    pub left_text: Entity,
    pub right_text: Entity,
    pub left_position: usize,
    pub right_position: usize,
    pub text_state: SplashScreenTextState,
}
