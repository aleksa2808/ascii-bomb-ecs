use bevy::prelude::*;

use crate::{loading::resources::LoadingContext, AppState};

use super::LoadingAssetHandles;

pub fn loading_update(
    mut commands: Commands,
    server: Res<AssetServer>,
    loading_asset_handles: Res<LoadingAssetHandles>,
    loading_context: Res<LoadingContext>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    use bevy::asset::LoadState;

    // iterate over all loading resources and check their state
    for handle in &loading_asset_handles.0 {
        match server
            .get_load_state(handle.id())
            .expect("asset marked as loading does not have loading info")
        {
            LoadState::Loaded => (),
            // if one of the assets is still loading then just skip until the next time this function is called
            LoadState::NotLoaded | LoadState::Loading => return,
            // situations where asset loading failed for some reason are not handled, just crash the app
            LoadState::Failed => panic!("failed to load an asset"),
        }
    }

    // when this point is reached, all assets are loaded, so teardown the current state context and move on to the next one
    next_state.set(loading_context.next_state);

    commands.remove_resource::<LoadingAssetHandles>();
    commands.remove_resource::<LoadingContext>();
}
