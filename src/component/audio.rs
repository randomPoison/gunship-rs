use ecs::*;
use engine::*;
use scene::Scene;
use std::rc::Rc;
use super::DefaultMessage;
use super::struct_component_manager::{Iter, IterMut, StructComponentManager};
use wav::Wave;

#[derive(Debug, Clone)]
pub struct AudioSource {
    audio_clip: Rc<Wave>,
    offset:     usize,
    is_playing: bool,
    looping:    bool,
}

impl AudioSource {
    /// Start playing the current audio clip from where it left off.
    pub fn play(&mut self) {
        self.is_playing = true;
    }

    /// Pause the clip without resetting it to the beginning.
    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    /// Stop the current audio clip and reset it to the beginning.
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.offset = 0;
    }

    /// Reset the audio clip the start without stoping it.
    pub fn reset(&mut self) {
        self.offset = 0;
    }

    /// Retrieve whether the audio clip is currently playing.
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }
}

impl Component for AudioSource {
    type Manager = AudioSourceManager;
    type Message = DefaultMessage<AudioSource>;
}

#[derive(Debug, Clone)]
pub struct AudioSourceManager(StructComponentManager<AudioSource>);

impl AudioSourceManager {
    pub fn new() -> AudioSourceManager {
        AudioSourceManager(StructComponentManager::new())
    }

    pub fn assign(&mut self, entity: Entity, clip_name: &str) -> &AudioSource {
        let audio_clip = Engine::resource_manager().get_audio_clip(clip_name);
        self.0.assign(entity, AudioSource {
            audio_clip: audio_clip,
            offset:     0,
            is_playing: false,
            looping:    false,
        })
    }

    pub fn get(&self, entity: Entity) -> Option<&AudioSource> {
        self.0.get(entity)
    }

    pub fn iter(&self) -> Iter<AudioSource> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<AudioSource> {
        self.0.iter_mut()
    }
}

impl ComponentManagerBase for AudioSourceManager {}

impl ComponentManager for AudioSourceManager {
    type Component = AudioSource;

    fn register(builder: &mut EngineBuilder) {
        let audio_manager = AudioSourceManager::new();
        builder.register_manager(audio_manager);
    }

    fn get(&self, entity: Entity) -> Option<&Self::Component> {
        self.0.get(entity)
    }

    fn destroy(&self, entity: Entity) {
        self.0.destroy(entity);
    }
}

derive_Singleton!(AudioSourceManager);

pub struct AudioSystem;

impl System for AudioSystem {
    fn update(&mut self, scene: &Scene, delta: f32) {
        let mut audio_source_manager = unsafe { scene.get_manager_mut::<AudioSourceManager>() }; // FIXME: Very bad, use new system.

        // TODO: Use a better method to filter out audio sources that aren't playing.
        for mut audio_source in audio_source_manager.iter_mut()
                                .map(|(audio_source, _)| audio_source)
                                .filter(|audio_source| audio_source.is_playing) {
            // Create an iterator over the samples using the data from the audio clip.
            let total_samples = {
                let mut stream = audio_source.audio_clip.data.samples[audio_source.offset..].iter()
                    .map(|sample| *sample);

                // Sream the samples to the audio card.
                let samples_written = scene.audio_source.stream(&mut stream, delta);

                // Determine if we're done playing the clip yet.
                audio_source.offset + samples_written
            };
            if total_samples >= audio_source.audio_clip.data.samples.len() {
                audio_source.offset = 0;

                if !audio_source.looping {
                    audio_source.stop();
                }
            } else {
                audio_source.offset = total_samples;
            }
        }
    }
}
