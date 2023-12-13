use std::sync::{Arc, RwLock};

use crate::{core::parametrs::ExponentialTimeParametr, utils::convert::{linear_to_voltage, linear_to_power}};

use super::SampleProcessor;

pub type TimeParametr = Arc<RwLock<ExponentialTimeParametr>>;

pub struct SampleDetector {
    attack: TimeParametr,
    release: TimeParametr,
    last_output: f32,
}

impl SampleDetector {
    pub fn new(attack: TimeParametr, release: TimeParametr) -> Self {
        Self {
            attack,
            release,
            last_output: 0.0,
        }
    }
}

impl SampleProcessor for SampleDetector {
    fn process(&mut self, sample: f32) -> f32 {
        let sample = sample * sample;
        let time = if sample > self.last_output {
            self.attack.read().unwrap().exponential_time
        } else {
            self.release.read().unwrap().exponential_time
        };
        self.last_output = time * (self.last_output - sample) + sample;
        if self.last_output <= 0.0 {
            -96.0
        } else {
            linear_to_power(self.last_output)
        }
    }
}
