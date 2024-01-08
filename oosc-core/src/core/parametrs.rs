use std::any::Any;

use crate::{
    error::Error,
    utils::{
        convert::{
            cents_to_freq_coefficient, exponential_time, octave_offset_to_notes, power_to_linear,
            split_bipolar_pan,
        },
        evaluate::{Modulation, Evaluate, ModulationContainer},
        math::clamp,
        Shared,
    },
};

pub trait Parametr<T>: Send + Sync
where
    T: Clone + PartialOrd + Default,
{
    fn set_value(&mut self, value: T);
    fn get_value(&self) -> T;
    fn range(&self) -> (T, T);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub type SharedParametr<T> = Shared<dyn Parametr<T>>;

pub struct ValueParametr<T>
where
    T: Clone,
{
    value: T,
    range: (T, T),
    modifiers: ModulationContainer,
}

impl<T> ValueParametr<T>
where
    T: Clone,
{
    pub fn new(value: T, range: (T, T)) -> Self {
        Self {
            value,
            range,
            modifiers: Default::default(),
        }
    }
}

impl ValueParametr<f32> {
    pub fn set_evaluate_range(&mut self, range: (f32, f32)) -> &mut Self {
        self.container_mut().modulation_range = range;
        self
    }
}

impl<T> Modulation for ValueParametr<T>
where
    T: Clone,
{
    fn container(&self) -> &ModulationContainer {
        &self.modifiers
    }

    fn container_mut(&mut self) -> &mut ModulationContainer {
        &mut self.modifiers
    }
}

impl<T> Evaluate<f32> for T
where
    T: Modulation + Parametr<f32> + Send + Sync,
{
    fn evaluate(&self, t: f32) -> Result<f32, Error> {
        self.container().evaluate(t)
    }
}

impl<T> Parametr<T> for ValueParametr<T>
where
    T: Clone + PartialOrd + Default + Send + Sync + 'static,
{
    fn set_value(&mut self, value: T) {
        self.value = clamp(value, &self.range);
    }

    fn get_value(&self) -> T {
        self.value.clone()
    }

    fn range(&self) -> (T, T) {
        self.range.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct OctaveParametr {
    pub notes: i32,
    parametr: ValueParametr<i32>,
}

impl OctaveParametr {
    pub fn new(parametr: ValueParametr<i32>) -> Self {
        Self {
            notes: octave_offset_to_notes(parametr.get_value()),
            parametr,
        }
    }
}

impl Parametr<i32> for OctaveParametr {
    fn set_value(&mut self, value: i32) {
        self.parametr.set_value(value);
        self.notes = octave_offset_to_notes(self.parametr.get_value());
    }

    fn get_value(&self) -> i32 {
        self.parametr.get_value()
    }

    fn range(&self) -> (i32, i32) {
        self.parametr.range()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct PanParametr {
    pub polar: (f32, f32),
    bipolar: ValueParametr<f32>,
}

impl PanParametr {
    pub fn new(parametr: ValueParametr<f32>) -> Self {
        Self {
            polar: split_bipolar_pan(parametr.get_value()),
            bipolar: parametr,
        }
    }

    pub fn evaluate_polar(&self, t: f32) -> Result<(f32, f32), Error> {
        if self.modulated() {
            Ok(split_bipolar_pan(self.evaluate(t)?))
        } else {
            Ok(self.polar)
        }
    }
}

impl From<ValueParametr<f32>> for PanParametr {
    fn from(parametr: ValueParametr<f32>) -> Self {
        Self::new(parametr)
    }
}

impl Parametr<f32> for PanParametr {
    fn set_value(&mut self, value: f32) {
        self.bipolar.set_value(value);
        self.polar = split_bipolar_pan(value);
    }

    fn get_value(&self) -> f32 {
        self.bipolar.get_value()
    }

    fn range(&self) -> (f32, f32) {
        self.bipolar.range()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Modulation for PanParametr {
    fn container(&self) -> &ModulationContainer {
        self.bipolar.container()
    }

    fn container_mut(&mut self) -> &mut ModulationContainer {
        self.bipolar.container_mut()
    }
}

impl Default for PanParametr {
    fn default() -> Self {
        Self::from(ValueParametr::new(0.0, (-1.0, 1.0)))
    }
}

pub struct VolumeParametr {
    pub linear: f32,
    db: ValueParametr<f32>,
}

impl VolumeParametr {
    pub fn new(parametr: ValueParametr<f32>) -> Self {
        Self {
            linear: power_to_linear(parametr.get_value()),
            db: parametr,
        }
    }

    pub fn evaluate_linear(&self, t: f32) -> Result<f32, Error> {
        Ok(power_to_linear(self.evaluate(t)?))
    }
}

impl From<ValueParametr<f32>> for VolumeParametr {
    fn from(parametr: ValueParametr<f32>) -> Self {
        Self::new(parametr)
    }
}

impl Parametr<f32> for VolumeParametr {
    fn set_value(&mut self, value: f32) {
        self.db.set_value(value);
        self.linear = power_to_linear(value);
    }

    fn get_value(&self) -> f32 {
        self.db.get_value()
    }

    fn range(&self) -> (f32, f32) {
        self.db.range()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Modulation for VolumeParametr {
    fn container(&self) -> &ModulationContainer {
        self.db.container()
    }

    fn container_mut(&mut self) -> &mut ModulationContainer {
        self.db.container_mut()
    }
}

impl Default for VolumeParametr {
    fn default() -> Self {
        Self::from(ValueParametr::new(0.0, (-96.0, 3.0)))
    }
}

pub struct ExponentialTimeParametr {
    pub exponential_time: f32,
    sample_rate: f32,
    linear_time: ValueParametr<f32>,
}

impl ExponentialTimeParametr {
    pub fn new(parametr: ValueParametr<f32>, sample_rate: f32) -> Self {
        Self {
            exponential_time: exponential_time(parametr.get_value(), sample_rate),
            linear_time: parametr,
            sample_rate,
        }
    }
}

impl Parametr<f32> for ExponentialTimeParametr {
    fn set_value(&mut self, value: f32) {
        self.linear_time.set_value(value);
        self.exponential_time = exponential_time(value, self.sample_rate);
    }

    fn get_value(&self) -> f32 {
        self.linear_time.get_value()
    }

    fn range(&self) -> (f32, f32) {
        self.linear_time.range()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Modulation for ExponentialTimeParametr {
    fn container(&self) -> &ModulationContainer {
        self.linear_time.container()
    }

    fn container_mut(&mut self) -> &mut ModulationContainer {
        self.linear_time.container_mut()
    }
}

pub struct CentsParametr {
    pub freq: f32,
    parametr: ValueParametr<i32>,
}

impl CentsParametr {
    pub fn new(parametr: ValueParametr<i32>) -> Self {
        Self {
            freq: cents_to_freq_coefficient(parametr.get_value() as f32),
            parametr,
        }
    }
}

impl Parametr<i32> for CentsParametr {
    fn set_value(&mut self, value: i32) {
        self.parametr.set_value(value);
        self.freq = cents_to_freq_coefficient(value as f32);
    }

    fn get_value(&self) -> i32 {
        self.parametr.get_value()
    }

    fn range(&self) -> (i32, i32) {
        self.parametr.range()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct CallbackParametr<S, G, R, T>
where
    S: FnMut(T) + Send + Sync,
    G: Fn() -> T + Send + Sync,
    R: Fn() -> (T, T) + Send + Sync,
    T: Clone + PartialOrd + Default + Send + Sync,
{
    pub setter: S,
    pub getter: G,
    pub range: R,
}

impl<S, G, R, T> CallbackParametr<S, G, R, T>
where
    S: FnMut(T) + Send + Sync + 'static,
    G: Fn() -> T + Send + Sync + 'static,
    R: Fn() -> (T, T) + Send + Sync + 'static,
    T: Clone + PartialOrd + Default + Send + Sync + 'static,
{
    pub fn new(setter: S, getter: G, range: R) -> Self {
        Self {
            setter,
            getter,
            range,
        }
    }
}

impl<S, G, R, T> Parametr<T> for CallbackParametr<S, G, R, T>
where
    S: FnMut(T) + Send + Sync + 'static,
    G: Fn() -> T + Send + Sync + 'static,
    R: Fn() -> (T, T) + Send + Sync + 'static,
    T: Clone + PartialOrd + Default + Send + Sync + 'static,
{
    fn set_value(&mut self, value: T) {
        (self.setter)(value)
    }

    fn get_value(&self) -> T {
        (self.getter)()
    }

    fn range(&self) -> (T, T) {
        (self.range)()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
