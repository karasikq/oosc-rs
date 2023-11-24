use crate::{error::Error, utils::sample_buffer::SampleBuffer};

pub trait Effect<'a> {
    fn process(&self, buffer: &mut SampleBuffer) -> Result<(), Error>;
}
