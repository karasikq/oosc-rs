use crate::{error::Error, utils::sample_buffer::SampleBuffer};

pub struct Amplifier {
    gain: f32,
    pan: (f32, f32),
}

impl Amplifier {
    pub fn new(gain: f32, pan: (f32, f32)) -> Self {
        Self { gain, pan }
    }

    pub fn process(&self, buffer: &mut SampleBuffer) -> Result<(), Error> {
        let gain = self.gain;
        let pan = self.pan;
        buffer.iter_mut(0)?.for_each(|s| *s *= pan.0 * gain);
        buffer.iter_mut(1)?.for_each(|s| *s *= pan.1 * gain);
        Ok(())
    }
}
