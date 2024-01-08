use crate::utils::{consts::PI_2M, evaluate::Evaluate, make_shared};

use super::{
    parametrs::{SharedParametr, ValueParametr},
    waveshape::WaveShape,
};

pub struct LFO {
    shape: WaveShape,
    frequency: SharedParametr<f32>,
}

impl LFO {
    pub fn new(shape: WaveShape, frequency: f32) -> Self {
        let frequency = make_shared(ValueParametr::new(frequency, (0.001, 20.0)));
        Self { shape, frequency }
    }
}

impl Evaluate<f32> for LFO {
    fn evaluate(&self, t: f32) -> Result<f32, crate::error::Error> {
        let freq = PI_2M * self.frequency.read().unwrap().get_value() * t;
        self.shape.evaluate(freq)
    }
}
