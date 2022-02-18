use std::io::Cursor;

use anyhow::Result;
use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    prelude::*,
    reflect as bevy_reflect,
    reflect::TypeUuid,
    utils::HashMap,
};
use itertools::Itertools;
use kira::{
    instance::{handle::InstanceHandle, InstanceLoopStart, InstanceSettings, StopInstanceSettings},
    manager::{AudioManager, AudioManagerSettings},
    sound::{handle::SoundHandle, Sound as KiraSound, SoundSettings},
    Frame, Value,
};
use rodio::{Decoder, Source};

use super::{Audio, AudioCommand, SoundHandles, SoundID, SoundLoader};

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "5dc1e69a-70a3-4c99-8d8b-0d2ac17906cc"]
pub struct Sound(KiraSound);

impl AssetLoader for SoundLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            let decoder = Decoder::new(Cursor::new(bytes)).unwrap();
            let sample_rate = decoder.sample_rate();
            let num_of_channels = decoder.channels();
            let data = decoder.convert_samples();
            let frames = match num_of_channels {
                1 => data
                    .into_iter()
                    .map(Frame::from_mono)
                    .collect::<Vec<Frame>>(),
                2 => data
                    .tuples()
                    .map(|(left, right)| Frame::new(left, right))
                    .collect::<Vec<Frame>>(),
                _ => unreachable!(),
            };

            let sound = KiraSound::from_frames(sample_rate, frames, SoundSettings::default());

            load_context.set_default_asset(LoadedAsset::new(Sound(sound)));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ogg", "wav"]
    }
}

pub struct AudioBackend {
    sound_id_handle_map: HashMap<SoundID, SoundHandle>,
    audio_manager: AudioManager,
    previous_instance: Option<InstanceHandle>,
    volume: f64,
}

impl Default for AudioBackend {
    fn default() -> Self {
        Self {
            sound_id_handle_map: HashMap::default(),
            audio_manager: AudioManager::new(AudioManagerSettings::default()).unwrap(),
            previous_instance: None,
            volume: 1.0,
        }
    }
}

pub fn play_queued_audio(
    audio: Res<Audio>,
    mut audio_backend: NonSendMut<AudioBackend>,
    sound_handles: Res<SoundHandles>,
    sounds: Res<Assets<Sound>>,
) {
    let mut volume_change = audio.volume_change.write();
    if let Some(new_volume) = *volume_change {
        let new_volume = new_volume.into();
        audio_backend.volume = new_volume;

        if let Some(ref mut instance) = &mut audio_backend.previous_instance {
            instance.set_volume(new_volume).unwrap();
        }

        *volume_change = None;
    }

    let mut audio_command = audio.audio_command.write();
    if let Some(command) = &*audio_command {
        let command_successful = match command {
            AudioCommand::Play { sound_id, looped } => {
                if let Some(ref mut previous_instance) = audio_backend.previous_instance {
                    previous_instance
                        .stop(StopInstanceSettings::default())
                        .unwrap();
                    audio_backend.previous_instance = None;
                }

                let sound_handle =
                    if let Some(handle) = audio_backend.sound_id_handle_map.get(sound_id) {
                        Some(handle.clone())
                    } else if let Some(sound) =
                        sounds.get(sound_handles.0.get(sound_id).unwrap().clone())
                    {
                        let sound_handle = audio_backend
                            .audio_manager
                            .add_sound(sound.0.clone())
                            .unwrap();

                        audio_backend
                            .sound_id_handle_map
                            .insert(*sound_id, sound_handle.clone());

                        Some(sound_handle)
                    } else {
                        // the sound has not yet been loaded
                        None
                    };

                if let Some(mut sound_handle) = sound_handle {
                    audio_backend.previous_instance = Some(
                        sound_handle
                            .play(InstanceSettings {
                                volume: Value::Fixed(audio_backend.volume),
                                loop_start: if *looped {
                                    InstanceLoopStart::Custom(0.0)
                                } else {
                                    InstanceLoopStart::None
                                },
                                ..Default::default()
                            })
                            .unwrap(),
                    );
                    true
                } else {
                    false
                }
            }
            AudioCommand::Stop => {
                if let Some(ref mut previous_instance) = audio_backend.previous_instance {
                    previous_instance
                        .stop(StopInstanceSettings::default())
                        .unwrap();
                    audio_backend.previous_instance = None;
                }

                true
            }
        };

        if command_successful {
            *audio_command = None;
        }
    }
}
