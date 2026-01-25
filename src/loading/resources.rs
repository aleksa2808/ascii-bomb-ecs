use bevy::{ecs as bevy_ecs, prelude::*};

use crate::AppState;

#[derive(Default, Resource)]
pub struct LoadingAssetHandles(pub Vec<UntypedHandle>);

#[derive(Resource)]
pub struct LoadingContext {
    pub next_state: AppState,
}
