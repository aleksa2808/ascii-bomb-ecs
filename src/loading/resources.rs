use bevy::prelude::*;

use crate::AppState;

#[derive(Default)]
pub struct AssetsLoading(pub Vec<HandleUntyped>);

pub struct LoadingContext {
    pub next_state: AppState,
}
