#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod web;

use std::sync::atomic::{AtomicUsize, Ordering};

use bevy::{ecs as bevy_ecs, prelude::*, utils::HashMap};
use parking_lot::RwLock;

#[cfg(target_arch = "wasm32")]
use crate::AppState;

#[cfg(not(target_arch = "wasm32"))]
use self::native::{play_queued_audio, AudioBackend, Sound};
#[cfg(target_arch = "wasm32")]
use self::web::{play_queued_audio, prepare_webaudio_buffers, AudioBackend, Sound};

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<Sound>()
            .init_asset_loader::<SoundLoader>()
            .init_resource::<SoundHandles>()
            .init_resource::<Audio>()
            .init_non_send_resource::<AudioBackend>()
            // TODO: this system should probably be placed after all other game systems, as it was before
            .add_systems((play_queued_audio, apply_system_buffers).chain());

        #[cfg(target_arch = "wasm32")]
        app.add_system_set(
            SystemSet::on_exit(AppState::Loading)
                .with_system(prepare_webaudio_buffers.exclusive_system()),
        );
    }
}

#[derive(Default)]
pub struct SoundLoader;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundID(usize);

static SOUND_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
impl SoundID {
    fn generate() -> Self {
        Self(SOUND_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Mapping between the `Sound` handles and the app-level unique IDs.
///
/// Needed because of the differences in how the native and the web build audio systems operate.
/// In the native build the IDs are used to obtain the `Sound` handles again, while in the
/// web build they are used to obtain the pre-loaded Web Audio API audio buffers.
#[derive(Default, Resource)]
pub struct SoundHandles(HashMap<SoundID, Handle<Sound>>);

impl SoundHandles {
    pub fn add_handle(&mut self, sound_handle: Handle<Sound>) -> SoundID {
        let sound_id = SoundID::generate();
        self.0.insert(sound_id, sound_handle);
        sound_id
    }
}

pub enum AudioCommand {
    Play { sound_id: SoundID, looped: bool },
    Stop,
}

/// A frontend interface resource that can be used to place audio requests from any thread.
#[derive(Default, Resource)]
pub struct Audio {
    pub audio_command: RwLock<Option<AudioCommand>>,
    pub volume_change: RwLock<Option<f32>>,
}

impl Audio {
    pub fn play(&self, sound_id: SoundID) {
        *self.audio_command.write() = Some(AudioCommand::Play {
            sound_id,
            looped: false,
        });
    }

    pub fn play_looped(&self, sound_id: SoundID) {
        *self.audio_command.write() = Some(AudioCommand::Play {
            sound_id,
            looped: true,
        });
    }

    pub fn stop(&self) {
        *self.audio_command.write() = Some(AudioCommand::Stop);
    }

    pub fn set_volume(&self, volume: f32) {
        *self.volume_change.write() = Some(volume);
    }
}
