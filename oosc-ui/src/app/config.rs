use cpal::StreamConfig;

#[derive(Copy, Clone)]
pub struct Config {
    pub channels: u32,
    pub sample_rate: u32,
    pub delta_time: f32,
    pub buffer_size: usize,
}

impl From<Config> for StreamConfig {
    fn from(val: Config) -> Self {
        StreamConfig {
            channels: val.channels as u16,
            sample_rate: cpal::SampleRate(val.sample_rate),
            buffer_size: cpal::BufferSize::Fixed(val.buffer_size as u32),
        }
    }
}
