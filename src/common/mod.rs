use bevy::prelude::*;

use self::{resources::*, systems::*};

pub mod constants;
pub mod resources;
mod systems;
pub mod types;

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BaseColorMaterials>()
            .init_resource::<GameOptionStore>()
            .init_resource::<PersistentHighScores>()
            .init_resource::<Fonts>()
            .add_startup_system(set_volume_based_on_options);
    }
}
