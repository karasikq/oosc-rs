use crate::{
    core::parametrs::{PanParametr, Parametr, ValueParametr, VolumeParametr},
    error::Error,
    utils::sample_buffer::SampleBuffer,
};

use super::{Effect, State};

pub struct Amplifier {
    gain: VolumeParametr,
    pan: PanParametr,
    state: State,
}

impl Amplifier {
    pub fn new(gain: VolumeParametr, pan: PanParametr, state: State) -> Self {
        Self { gain, pan, state }
    }

    pub fn volume(&mut self) -> &mut impl Parametr<f32> {
        &mut self.gain
    }

    pub fn pan(&mut self) -> &mut impl Parametr<f32> {
        &mut self.pan
    }
}

impl Effect for Amplifier {
    fn process(&mut self, buffer: &mut SampleBuffer) -> Result<(), Error> {
        let gain = &self.gain;
        let pan = &self.pan;
        buffer.iter_buffers().enumerate().for_each(|(i, buffer)| {
            let pan = match i {
                0 => pan.polar.0,
                1 => pan.polar.1,
                _ => 1.0,
            };
            buffer.iter_mut().for_each(|s| *s *= pan * gain.linear);
        });
        Ok(())
    }

    fn state(&self) -> State {
        self.state
    }

    fn set_state(&mut self, state: State) {
        self.state = state;
    }
}

impl Default for Amplifier {
    fn default() -> Self {
        let gain = VolumeParametr::from(ValueParametr::new(3.0, (-96.0, 3.0)));
        let pan = PanParametr::from(ValueParametr::new(0.0, (-1.0, 1.0)));
        Self::new(gain, pan, State::Enabled)
    }
}
