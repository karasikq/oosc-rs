use crate::error::Error;
use crate::utils::Shared;

use super::interpolation::interpolate_linear;

pub trait Evaluate<T>: Send + Sync {
    fn evaluate(&self, t: f32) -> Result<T, Error>;
    fn evaluate_mut(&mut self, t: f32) -> Result<T, Error> {
        self.evaluate(t)
    }
}

pub struct ModulationContainer {
    pub modulators: Vec<Shared<dyn Evaluate<f32>>>,
    pub modulation_range: (f32, f32),
    pub time: f32,
}

pub trait Modulation {
    fn modulated(&self) -> bool {
        !self.container().modulators.is_empty()
    }
    fn container(&self) -> &ModulationContainer;
    fn container_mut(&mut self) -> &mut ModulationContainer;
    fn next_value(&mut self, delta_time: f32) -> Result<(), Error> {
        self.container_mut().next_value(delta_time)?;
        Ok(())
    }
}

impl ModulationContainer {
    pub fn new() -> Self {
        Self {
            modulators: Vec::new(),
            modulation_range: (0.0, 0.0),
            time: 0.0,
        }
    }

    pub fn range(self, range: (f32, f32)) -> Self {
        Self {
            modulation_range: range,
            ..self
        }
    }

    pub fn next_value(&mut self, delta_time: f32) -> Result<f32, Error> {
        let evaluate = self.evaluate(self.time)?;
        self.time += delta_time;
        Ok(evaluate)
    }
}

impl Default for ModulationContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluate<f32> for ModulationContainer {
    fn evaluate(&self, t: f32) -> Result<f32, Error> {
        let mut mod_result = 1.0;
        for m in self.modulators.iter() {
            mod_result *= m.read().unwrap().evaluate(t)?;
        }
        Ok(interpolate_linear(
            self.modulation_range.0,
            self.modulation_range.1,
            mod_result,
        ))
    }
}
