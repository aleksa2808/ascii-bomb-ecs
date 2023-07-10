use bevy::prelude::*;

use crate::AppState;

use self::{resources::*, systems::*};

pub mod resources;
mod systems;

pub struct LoadingPlugin {
    pub loading_state: AppState,
    pub next_state: AppState,
}

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetsLoading>()
            .insert_resource(LoadingContext {
                next_state: self.next_state,
            })
            .add_systems(
                (loading_update, apply_system_buffers)
                    .chain()
                    .in_set(OnUpdate(self.loading_state)),
            );
    }
}
