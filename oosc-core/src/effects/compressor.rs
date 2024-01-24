use std::sync::{Arc, RwLock};

use crate::core::parameter::SharedParameter;
use crate::effects::Effect;
use crate::utils::convert::power_to_linear;
use crate::utils::sample_buffer::BufferSettings;
use crate::{
    core::parameter::{ExponentialTimeParameter, Parameter, ValueParameter, VolumeParameter},
    error::Error,
    utils::sample_buffer::SampleBuffer,
};

use super::sample_detector::{SampleDetector, TimeParametr};
use super::{SampleProcessor, State};

pub enum KneeType {
    Soft(VolumeParameter),
    Hard,
}

pub struct Compressor {
    threshold: VolumeParameter,
    ratio: ValueParameter<f32>,
    knee_type: KneeType,
    attack: TimeParametr,
    release: TimeParametr,
    detectors: Vec<SampleDetector>,
    state: State,
}

impl Compressor {
    pub fn new(
        threshold: VolumeParameter,
        ratio: ValueParameter<f32>,
        knee_type: KneeType,
        attack: ExponentialTimeParameter,
        release: ExponentialTimeParameter,
        channels: usize,
        state: State,
    ) -> Self {
        let attack = Arc::new(RwLock::new(attack));
        let release = Arc::new(RwLock::new(release));
        let detectors = (0..channels)
            .map(|_| SampleDetector::new(attack.clone(), release.clone()))
            .collect();

        Self {
            threshold,
            ratio,
            knee_type,
            attack,
            release,
            detectors,
            state,
        }
    }

    pub fn attack(&self) -> SharedParameter<f32> {
        self.attack.clone()
    }

    pub fn release(&self) -> SharedParameter<f32> {
        self.release.clone()
    }

    pub fn default(settings: &BufferSettings) -> Self {
        let threshold = VolumeParameter::new(ValueParameter::new(-3.0, (-96.0, 0.0)));
        let ratio = ValueParameter::new(50.0, (1.0, 100.0));
        let knee_type = KneeType::Soft(VolumeParameter::new(ValueParameter::new(6.0, (0.0, 36.0))));
        let attack = ExponentialTimeParameter::new(
            ValueParameter::new(0.005, (0.001, 0.5)),
            settings.sample_rate,
        );
        let release = ExponentialTimeParameter::new(
            ValueParameter::new(0.005, (0.001, 5.0)),
            settings.sample_rate,
        );

        Self::new(
            threshold,
            ratio,
            knee_type,
            attack,
            release,
            settings.channels,
            State::Enabled,
        )
    }

    fn proccess_sample(&mut self, sample: &mut f32, processor: usize) {
        let detected = self.detectors.get_mut(processor).unwrap().process(*sample);
        let threshold = self.threshold.get_value();
        let ratio = self.ratio.get_value();
        let output = match &self.knee_type {
            KneeType::Soft(width) => {
                let width = width.get_value();
                let region = 2.0 * (detected - threshold);
                if region < -width {
                    detected
                } else if region > width {
                    threshold + (detected - threshold) / ratio
                } else {
                    detected
                        + ((1.0 / (ratio - 1.0)) * (detected - threshold + width * 0.5).powi(2)
                            / (2.0 * width))
                }
            }
            KneeType::Hard => {
                if detected <= threshold {
                    detected
                } else {
                    threshold + (detected - threshold) / ratio
                }
            }
        };
        *sample *= power_to_linear(output - detected);
    }
}

impl Effect for Compressor {
    fn process(&mut self, buffer: &mut SampleBuffer) -> Result<(), Error> {
        buffer.iter_buffers().enumerate().for_each(|(i, buffer)| {
            buffer
                .iter_mut()
                .for_each(|sample| self.proccess_sample(sample, i));
        });
        Ok(())
    }

    fn state(&self) -> State {
        self.state
    }

    fn set_state(&mut self, state: State) {
        self.state = state;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
