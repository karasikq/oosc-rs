#[derive(Copy, Clone)]
pub struct Config {
    pub channels: u32,
    pub sample_rate: u32,
    pub delta_time: f32,
    pub buffer_size: usize,
}
