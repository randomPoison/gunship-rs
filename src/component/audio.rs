use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use scene::Scene;
use ecs::{Entity, ComponentManager, System};
use resource::ResourceManager;
use wav::Wave;

pub struct AudioSource {
    wave:   Rc<Wave>,
    offset: usize,
}

pub struct AudioSourceManager {
    resource_manager: Rc<RefCell<ResourceManager>>,
    audio_sources:    Vec<AudioSource>,
    entities:         Vec<Entity>,
    indices:          HashMap<Entity, usize>,
}

impl AudioSourceManager {
    pub fn new(resource_manager: Rc<RefCell<ResourceManager>>) -> AudioSourceManager {
        AudioSourceManager {
            resource_manager: resource_manager,
            audio_sources:    Vec::new(),
            entities:         Vec::new(),
            indices:          HashMap::new(),
        }
    }

    pub fn assign(&mut self, entity: Entity, clip_name: &str) -> &AudioSource {
        assert!(!self.indices.contains_key(&entity));

        let mut resource_manager = self.resource_manager.borrow_mut();
        let wave = resource_manager.get_audio_clip(clip_name);
        let index = self.audio_sources.len();
        // let () = Box::new(wave.data.samples.iter().map(|sample| *sample));
        self.audio_sources.push(AudioSource {
            wave: wave,
            offset: 0,
        });
        self.entities.push(entity);
        self.indices.insert(entity, index);

        &self.audio_sources[index]
    }
}

impl ComponentManager for AudioSourceManager {
}

pub struct AudioSystem;

impl System for AudioSystem {
    fn update(&mut self, scene: &mut Scene, _: f32) {
        let mut audio_handle = scene.get_manager::<AudioSourceManager>();
        let mut audio_source_manager = audio_handle.get();

        let mut audio_sources = &mut audio_source_manager.audio_sources;
        for audio_source in audio_sources.iter_mut() {
            // Create an iterator over the samples using the data from the audio clip.
            let mut stream = audio_source.wave.data.samples.iter()
                .skip(audio_source.offset)
                .map(|sample| *sample);

            // Sream the samples to the audio card.
            let samples_written = scene.audio_source.stream(&mut stream, 1.0);

            // Determine if we're done playing the clip yet.
            let total_samples = audio_source.offset + samples_written;
            if total_samples >= audio_source.wave.data.samples.len() {
                // TODO: Handle the audio clip finishing.
                audio_source.offset = 0;
            } else {
                audio_source.offset = total_samples;
            }
        }
    }
}
