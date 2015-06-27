#[derive(Debug, Clone)]
pub struct AudioSource;

impl AudioSource {
    pub fn stream<T: Iterator<Item = u16>>(&self, _data_source: &mut T, _max_time: f32) -> usize {
        0
    }
}

pub fn init() -> Result<AudioSource, String> {
    println!("bootstrap_audio::init() has not been implemented yet for linux");
    Ok(AudioSource)
}
