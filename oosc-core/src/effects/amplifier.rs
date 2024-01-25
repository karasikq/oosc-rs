use crate::{
    core::parameter::{
        NamedParameter, NamedParametersContainer, PanParameter, ValueParameter,
        VolumeParameter,
    },
    error::Error,
    utils::{make_shared, sample_buffer::SampleBuffer, Shared},
};

use super::{Effect, State};

pub struct Amplifier {
    gain: Shared<VolumeParameter>,
    pan: Shared<PanParameter>,
    parameters_f32: Vec<NamedParameter<f32>>,
    state: State,
}

impl Amplifier {
    pub fn new(gain: VolumeParameter, pan: PanParameter, state: State) -> Self {
        let gain = make_shared(gain);
        let pan = make_shared(pan);
        let parameters_f32 = vec![
            NamedParameter::new(gain.clone(), "Gain"),
            NamedParameter::new(pan.clone(), "Pan"),
        ];
        Self {
            gain,
            pan,
            state,
            parameters_f32,
        }
    }

    pub fn volume(&self) -> Shared<VolumeParameter> {
        self.gain.clone()
    }

    pub fn pan(&self) -> Shared<PanParameter> {
        self.pan.clone()
    }
}

impl Effect for Amplifier {
    fn state(&self) -> State {
        self.state
    }

    fn set_state(&mut self, state: State) {
        self.state = state;
    }

    fn process(&mut self, buffer: &mut SampleBuffer) -> Result<(), Error> {
        let gain = self.gain.read().unwrap().linear;
        let pan = self.pan.read().unwrap().polar;
        buffer.iter_buffers().enumerate().for_each(|(i, buffer)| {
            let pan = match i {
                0 => pan.0,
                1 => pan.1,
                _ => 1.0,
            };
            buffer.iter_mut().for_each(|s| *s *= pan * gain);
        });
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn parameters(&mut self) -> Option<&mut dyn NamedParametersContainer> {
        Some(self)
    }
}

impl Default for Amplifier {
    fn default() -> Self {
        let gain = VolumeParameter::from(ValueParameter::new(3.0, (-96.0, 3.0)));
        let pan = PanParameter::from(ValueParameter::new(0.0, (-1.0, 1.0)));
        Self::new(gain, pan, State::Enabled)
    }
}

impl NamedParametersContainer for Amplifier {
    fn name(&self) -> Option<&'static str> {
        Some("Amplifier")
    }

    fn parameters_f32(&self) -> Option<&[NamedParameter<f32>]> {
        Some(&self.parameters_f32)
    }
}
