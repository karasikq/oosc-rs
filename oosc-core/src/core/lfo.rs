use crate::utils::{consts::PI_2M, evaluate::Evaluate, make_shared};

use super::{
    parameter::{SharedParameter, ValueParameter},
    waveshape::WaveShape,
};

pub struct LFO {
    shape: WaveShape,
    frequency: SharedParameter<f32>,
}

impl LFO {
    pub fn new(shape: WaveShape, frequency: f32) -> Self {
        let frequency = make_shared(ValueParameter::new(frequency, (0.001, 20.0)));
        Self { shape, frequency }
    }
}

impl Evaluate<f32> for LFO {
    fn evaluate(&self, t: f32) -> Result<f32, crate::error::Error> {
        let freq = PI_2M * self.frequency.read().unwrap().get_value() * t;
        Ok(0.5 + self.shape.evaluate(freq)? * 0.5)
    }
}
