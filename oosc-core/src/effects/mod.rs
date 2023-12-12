use crate::{error::Error, utils::sample_buffer::SampleBuffer};

pub mod amplifier;
pub mod compressor;
pub mod sample_detector;
pub mod delay;

pub trait Effect {
    fn process(&mut self, buffer: &mut SampleBuffer) -> Result<(), Error>;
}

pub trait SampleProcessor {
    fn process(&mut self, sample: f32) -> f32;
}
