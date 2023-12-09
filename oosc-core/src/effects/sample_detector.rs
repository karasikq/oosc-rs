use crate::{core::parametrs::ExponentialTimeParametr, utils::convert::linear_to_voltage};

use super::SampleProcessor;

pub struct SampleDetector {
    attack: ExponentialTimeParametr,
    release: ExponentialTimeParametr,
    sample_rate: f32,
    last_output: f32,
}

impl SampleDetector {
    pub fn new(
        attack: ExponentialTimeParametr,
        release: ExponentialTimeParametr,
        sample_rate: f32,
    ) -> Self {
        Self {
            attack,
            release,
            sample_rate,
            last_output: 0.0,
        }
    }
}

impl SampleProcessor for SampleDetector {
    fn process(&mut self, sample: f32) -> f32 {
        let sample = sample * sample;
        let time = if sample > self.last_output {
            self.attack.exponential_time
        } else {
            self.release.exponential_time
        };
        self.last_output = time * (self.last_output - sample) + sample;
        if self.last_output <= 0.0 {
            -96.0
        } else {
            linear_to_voltage(self.last_output)
        }
    }
}
