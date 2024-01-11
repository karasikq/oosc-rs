use crate::error::Error;

pub mod stream_callback;
pub mod stream_renderer;

pub trait StreamCallback: Send + Sync {
    fn process_stream(&mut self, data: &mut [f32], time: f32, sample_rate: f32) -> Result<(), Error>;
}
