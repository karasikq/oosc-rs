use crate::{error::Error, utils::sample_buffer::{SampleBuffer, SampleBufferMono}};

pub mod amplifier;
pub mod compressor;
pub mod sample_detector;
pub mod chorus;
pub mod delay;

#[derive(Clone, Copy)]
pub enum State {
    Enabled,
    Disabled,
}

pub trait Effect {
    fn state(&self) -> State;
    fn set_state(&mut self, state: State);
    fn process(&mut self, buffer: &mut SampleBuffer) -> Result<(), Error>;
}

pub trait SampleProcessor {
    fn process(&mut self, sample: f32) -> f32;
}

pub trait MonoBufferProcessor {
    fn process(&mut self, buffer: &mut SampleBufferMono);
}
