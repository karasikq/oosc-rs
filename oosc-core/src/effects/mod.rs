use std::any::Any;

use crate::{
    core::parameter::NamedParametersContainer,
    error::Error,
    utils::sample_buffer::{SampleBuffer, SampleBufferMono},
};

pub mod amplifier;
pub mod chorus;
pub mod compressor;
pub mod delay;
pub mod filter;
pub mod sample_detector;

#[derive(Clone, Copy)]
pub enum State {
    Enabled,
    Disabled,
}

pub trait Effect: Send + Sync {
    fn state(&self) -> State;
    fn set_state(&mut self, state: State);
    fn process(&mut self, size: usize, buffer: &mut SampleBuffer) -> Result<(), Error>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    // Really need macro for this
    fn parameters(&mut self) -> Option<&mut dyn NamedParametersContainer> {
        None
    }
}

pub trait SampleProcessor {
    fn process(&mut self, sample: f32) -> f32;
}

pub trait MonoBufferProcessor {
    fn process(&mut self, size: usize, buffer: &mut SampleBufferMono);
}
