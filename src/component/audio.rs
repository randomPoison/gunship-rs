use std::collections::HashMap;
use std::rc::Rc;
use std::cell::{RefCell, Ref, RefMut};

use scene::Scene;
use ecs::{Entity, ComponentManager, System};
use resource::ResourceManager;
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

pub struct AudioSourceManager {
    resource_manager: Rc<ResourceManager>,
    audio_sources:    Vec<RefCell<AudioSource>>,
    entities:         Vec<Entity>,
    indices:          HashMap<Entity, usize>,
}

impl AudioSourceManager {
    pub fn new(resource_manager: Rc<ResourceManager>) -> AudioSourceManager {
        AudioSourceManager {
            resource_manager: resource_manager,
            audio_sources:    Vec::new(),
            entities:         Vec::new(),
            indices:          HashMap::new(),
        }
    }

    pub fn clone(&self, resource_manager: Rc<ResourceManager>) -> AudioSourceManager {
        AudioSourceManager {
            resource_manager: resource_manager,
            audio_sources:    self.audio_sources.clone(),
            entities:         self.entities.clone(),
            indices:          self.indices.clone(),
        }
    }

    pub fn assign(&mut self, entity: Entity, clip_name: &str) -> RefMut<AudioSource> {
        assert!(!self.indices.contains_key(&entity));

        let audio_clip = self.resource_manager.get_audio_clip(clip_name);
        let index = self.audio_sources.len();
        self.audio_sources.push(RefCell::new(AudioSource {
            audio_clip: audio_clip,
            offset:     0,
            is_playing: false,
            looping:    false,
        }));
        self.entities.push(entity);
        self.indices.insert(entity, index);

        self.audio_sources[index].borrow_mut()
    }

    pub fn get(&mut self, entity: Entity) -> Ref<AudioSource> {
        assert!(self.indices.contains_key(&entity));

        let index = *self.indices.get(&entity).unwrap();
        self.audio_sources[index].borrow()
    }

    pub fn get_mut(&self, entity: Entity) -> RefMut<AudioSource> {
        assert!(self.indices.contains_key(&entity));

        let index = *self.indices.get(&entity).unwrap();
        self.audio_sources[index].borrow_mut()
    }
}

impl ComponentManager for AudioSourceManager {
    fn destroy_all(&self, _entity: Entity) {
        println!("WARNING: AudioSourceManager.destroy_all() is not yet implemented");
    }

    fn destroy_marked(&mut self) {
    }
}

pub struct AudioSystem;

impl System for AudioSystem {
    fn update(&mut self, scene: &Scene, delta: f32) {
        let audio_source_manager = scene.get_manager::<AudioSourceManager>();

        // TODO: Use a better method to filter out audio sources that aren't playing.
        for mut audio_source in audio_source_manager.audio_sources.iter()
                                .map(|ref_cell| ref_cell.borrow_mut())
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
