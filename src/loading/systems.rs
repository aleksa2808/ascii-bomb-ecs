use bevy::prelude::*;

use crate::{loading::resources::LoadingContext, AppState};

use super::resources::AssetsLoading;

pub fn loading_update(
    mut commands: Commands,
    server: Res<AssetServer>,
    loading: Res<AssetsLoading>,
    loading_context: Res<LoadingContext>,
    mut state: ResMut<State<AppState>>,
) {
    use bevy::asset::LoadState;

    match server.get_group_load_state(loading.0.iter().map(|h| h.id)) {
        LoadState::Failed => panic!("failed to load an asset"),
        LoadState::Loaded => {
            // all assets are now ready
            state.replace(loading_context.next_state).unwrap();

            commands.remove_resource::<AssetsLoading>();
            commands.remove_resource::<LoadingContext>();
        }
        _ => {
            // NotLoaded/Loading: not fully ready yet
        }
    }
}
