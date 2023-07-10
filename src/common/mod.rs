use bevy::{ecs as bevy_ecs, prelude::*};

use self::{resources::*, systems::*};

pub mod constants;
pub mod resources;
pub mod systems;
pub mod types;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum Label {
    InputMapping,
}

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputActionStatusTracker>()
            .init_resource::<GameOptionStore>()
            .init_resource::<PersistentHighScores>()
            .init_resource::<Fonts>()
            .add_startup_system(set_volume_based_on_options)
            .add_systems(
                (clear_inputs, apply_system_buffers)
                    .chain()
                    .before(Label::InputMapping),
            )
            .add_systems(
                (handle_keyboard_input, apply_system_buffers)
                    .chain()
                    .in_set(Label::InputMapping),
            );
    }
}
