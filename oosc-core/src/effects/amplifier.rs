use crate::{
    core::parametrs::{PanParametr, VolumeParametr},
    error::Error,
    utils::sample_buffer::SampleBuffer,
};

use super::effect::Effect;

pub struct Amplifier {
    gain: VolumeParametr,
    pan: PanParametr,
}

impl Amplifier {
    pub fn new(gain: VolumeParametr, pan: PanParametr) -> Self {
        Self { gain, pan }
    }
}

impl<'a> Effect<'a> for Amplifier {
    fn process(&self, buffer: &mut SampleBuffer) -> Result<(), Error> {
        let gain = &self.gain;
        let pan = &self.pan;
        buffer.iter_mut(0)?.for_each(|s| *s *= pan.polar.0 * gain.linear);
        buffer.iter_mut(1)?.for_each(|s| *s *= pan.polar.1 * gain.linear);
        Ok(())
    }
}
