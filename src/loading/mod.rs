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
        app.init_resource::<LoadingAssetHandles>()
            .insert_resource(LoadingContext {
                next_state: self.next_state,
            })
            .add_systems(Update, loading_update.run_if(in_state(self.loading_state)));
    }
}
