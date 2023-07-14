use std::io::Cursor;

use anyhow::Result;
use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    prelude::*,
    reflect::{self as bevy_reflect, TypePath, TypeUuid},
    utils::HashMap,
};
use rodio::{Decoder, Source};
use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioContext, GainNode};

use super::{Audio, AudioCommand, SoundHandles, SoundID, SoundLoader};

#[derive(Debug, Clone, TypeUuid, TypePath)]
#[uuid = "1fd07a42-d528-40e0-b5ff-3be79a9400b0"]
pub struct Sound {
    num_of_channels: u32,
    sample_rate: f32,
    channel_data: Vec<Vec<f32>>,
}

impl AssetLoader for SoundLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            let decoder = Decoder::new(Cursor::new(bytes)).unwrap();
            let sample_rate = decoder.sample_rate() as f32;
            let num_of_channels = decoder.channels() as u32;
            let mut channel_data: Vec<Vec<f32>> = vec![vec![]; num_of_channels as usize];
            decoder
                .convert_samples()
                .zip((0..num_of_channels as usize).cycle())
                .for_each(|(f, i)| channel_data[i].push(f));

            load_context.set_default_asset(LoadedAsset::new(Sound {
                num_of_channels,
                sample_rate,
                channel_data,
            }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ogg", "wav"]
    }
}

pub struct AudioBackend {
    pub sound_id_audio_buffer_map: HashMap<SoundID, AudioBuffer>,
    pub audio_context: AudioContext,
    pub gain_node: GainNode,
    pub previous_source: Option<AudioBufferSourceNode>,
}

impl Default for AudioBackend {
    fn default() -> Self {
        let audio_context = AudioContext::new().unwrap();
        let gain_node = audio_context.create_gain().unwrap();
        gain_node
            .connect_with_audio_node(&audio_context.destination())
            .unwrap();

        Self {
            audio_context,
            gain_node,
            sound_id_audio_buffer_map: HashMap::default(),
            previous_source: None,
        }
    }
}

pub fn prepare_webaudio_buffers(
    mut commands: Commands,
    sound_handles: Res<SoundHandles>,
    mut audio_backend: NonSendMut<AudioBackend>,
    sounds: Res<Assets<Sound>>,
) {
    for (sound_id, sound_handle) in &sound_handles.0 {
        let Sound {
            num_of_channels: channels,
            sample_rate,
            channel_data,
        } = sounds.get(&sound_handle.clone()).unwrap();

        let buffer = audio_backend
            .audio_context
            .create_buffer(
                *channels,
                channel_data.iter().fold(0, |acc, v| acc + v.len() as u32),
                *sample_rate,
            )
            .unwrap();

        for (i, channel) in channel_data.iter().enumerate() {
            buffer.copy_to_channel(channel, i as i32).unwrap();
        }

        audio_backend
            .sound_id_audio_buffer_map
            .insert(*sound_id, buffer);
    }

    commands.remove_resource::<SoundHandles>();
}

pub fn play_queued_audio(audio: Res<Audio>, mut audio_backend: NonSendMut<AudioBackend>) {
    let mut volume_change = audio.volume_change.write();
    if let Some(new_volume) = &*volume_change {
        audio_backend.gain_node.gain().set_value(*new_volume);
        *volume_change = None;
    }

    let mut audio_command = audio.audio_command.write();
    if let Some(command) = &*audio_command {
        match command {
            AudioCommand::Play { sound_id, looped } => {
                if let Some(ref previous_source) = audio_backend.previous_source {
                    previous_source.stop().unwrap();
                }

                let music_source = audio_backend.audio_context.create_buffer_source().unwrap();
                music_source.set_buffer(Some(
                    audio_backend
                        .sound_id_audio_buffer_map
                        .get(sound_id)
                        .unwrap(),
                ));
                music_source
                    .connect_with_audio_node(&audio_backend.gain_node)
                    .unwrap();
                if *looped {
                    music_source.set_loop(true);
                }
                music_source.start().unwrap();

                audio_backend.previous_source = Some(music_source);
            }
            AudioCommand::Stop => {
                if let Some(ref previous_source) = audio_backend.previous_source {
                    previous_source.stop().unwrap();
                    audio_backend.previous_source = None;
                }
            }
        }

        *audio_command = None;
    }
}
