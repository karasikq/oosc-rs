use std::any::Any;

use crate::{
    error::Error,
    utils::{
        convert::{
            cents_to_freq_coefficient, exponential_time, octave_offset_to_notes, power_to_linear,
            split_bipolar_pan,
        },
        evaluate::{Evaluate, Modulation, ModulationContainer},
        math::clamp,
        Shared,
    },
};

pub trait Parameter<T>: Send + Sync
where
    T: Clone + PartialOrd + Default,
{
    fn set_value(&mut self, value: T);
    fn get_value(&self) -> T;
    fn range(&self) -> (T, T);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub type SharedParameter<T> = Shared<dyn Parameter<T>>;

pub struct ValueParameter<T>
where
    T: Clone,
{
    value: T,
    range: (T, T),
    modifiers: ModulationContainer,
}

impl<T> ValueParameter<T>
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

impl ValueParameter<f32> {
    pub fn set_evaluate_range(&mut self, range: (f32, f32)) -> &mut Self {
        self.container_mut().modulation_range = range;
        self
    }
}

impl Modulation for ValueParameter<f32> {
    fn container(&self) -> &ModulationContainer {
        &self.modifiers
    }

    fn container_mut(&mut self) -> &mut ModulationContainer {
        &mut self.modifiers
    }

    fn next_value(&mut self, delta_time: f32) -> Result<(), Error> {
        if self.modulated() {
            let value = self.container_mut().next_value(delta_time)?;
            self.set_value(value);
        }
        Ok(())
    }
}

impl<T> Evaluate<f32> for T
where
    T: Modulation + Parameter<f32> + Send + Sync,
{
    fn evaluate(&self, t: f32) -> Result<f32, Error> {
        self.container().evaluate(t)
    }
}

impl<T> Parameter<T> for ValueParameter<T>
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

pub struct OctaveParameter {
    pub notes: i32,
    parametr: ValueParameter<i32>,
}

impl OctaveParameter {
    pub fn new(parametr: ValueParameter<i32>) -> Self {
        Self {
            notes: octave_offset_to_notes(parametr.get_value()),
            parametr,
        }
    }
}

impl Parameter<i32> for OctaveParameter {
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

pub struct PanParameter {
    pub polar: (f32, f32),
    bipolar: ValueParameter<f32>,
}

impl PanParameter {
    pub fn new(parametr: ValueParameter<f32>) -> Self {
        Self {
            polar: split_bipolar_pan(parametr.get_value()),
            bipolar: parametr,
        }
    }
}

impl From<ValueParameter<f32>> for PanParameter {
    fn from(parametr: ValueParameter<f32>) -> Self {
        Self::new(parametr)
    }
}

impl Parameter<f32> for PanParameter {
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

impl Modulation for PanParameter {
    fn container(&self) -> &ModulationContainer {
        self.bipolar.container()
    }

    fn container_mut(&mut self) -> &mut ModulationContainer {
        self.bipolar.container_mut()
    }

    fn next_value(&mut self, delta_time: f32) -> Result<(), Error> {
        if self.modulated() {
            self.bipolar.next_value(delta_time)?;
            self.polar = split_bipolar_pan(self.bipolar.value);
        }
        Ok(())
    }
}

impl Default for PanParameter {
    fn default() -> Self {
        Self::from(ValueParameter::new(0.0, (-1.0, 1.0)))
    }
}

pub struct VolumeParameter {
    pub linear: f32,
    db: ValueParameter<f32>,
}

impl VolumeParameter {
    pub fn new(parametr: ValueParameter<f32>) -> Self {
        Self {
            linear: power_to_linear(parametr.get_value()),
            db: parametr,
        }
    }

    pub fn evaluate_linear(&self, t: f32) -> Result<f32, Error> {
        Ok(power_to_linear(self.evaluate(t)?))
    }
}

impl From<ValueParameter<f32>> for VolumeParameter {
    fn from(parametr: ValueParameter<f32>) -> Self {
        Self::new(parametr)
    }
}

impl Parameter<f32> for VolumeParameter {
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

impl Modulation for VolumeParameter {
    fn container(&self) -> &ModulationContainer {
        self.db.container()
    }

    fn container_mut(&mut self) -> &mut ModulationContainer {
        self.db.container_mut()
    }

    fn next_value(&mut self, delta_time: f32) -> Result<(), Error> {
        if self.modulated() {
            self.db.next_value(delta_time)?;
            self.linear = power_to_linear(self.db.value);
        }
        Ok(())
    }
}

impl Default for VolumeParameter {
    fn default() -> Self {
        Self::from(ValueParameter::new(0.0, (-96.0, 3.0)))
    }
}

pub struct ExponentialTimeParameter {
    pub exponential_time: f32,
    sample_rate: f32,
    linear_time: ValueParameter<f32>,
}

impl ExponentialTimeParameter {
    pub fn new(parametr: ValueParameter<f32>, sample_rate: f32) -> Self {
        Self {
            exponential_time: exponential_time(parametr.get_value(), sample_rate),
            linear_time: parametr,
            sample_rate,
        }
    }
}

impl Parameter<f32> for ExponentialTimeParameter {
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

impl Modulation for ExponentialTimeParameter {
    fn container(&self) -> &ModulationContainer {
        self.linear_time.container()
    }

    fn container_mut(&mut self) -> &mut ModulationContainer {
        self.linear_time.container_mut()
    }

    fn next_value(&mut self, delta_time: f32) -> Result<(), Error> {
        if self.modulated() {
            self.linear_time.next_value(delta_time)?;
            self.exponential_time = exponential_time(self.linear_time.value, self.sample_rate);
        }
        Ok(())
    }
}

pub struct CentsParameter {
    pub freq: f32,
    parameter: ValueParameter<f32>,
}

impl CentsParameter {
    pub fn new(parametr: ValueParameter<f32>) -> Self {
        Self {
            freq: cents_to_freq_coefficient(parametr.get_value()),
            parameter: parametr,
        }
    }
}

impl Parameter<f32> for CentsParameter {
    fn set_value(&mut self, value: f32) {
        self.parameter.set_value(value);
        self.freq = cents_to_freq_coefficient(value);
    }

    fn get_value(&self) -> f32 {
        self.parameter.get_value()
    }

    fn range(&self) -> (f32, f32) {
        self.parameter.range()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Modulation for CentsParameter {
    fn container(&self) -> &ModulationContainer {
        self.parameter.container()
    }

    fn container_mut(&mut self) -> &mut ModulationContainer {
        self.parameter.container_mut()
    }

    fn next_value(&mut self, delta_time: f32) -> Result<(), Error> {
        if self.modulated() {
            self.parameter.next_value(delta_time)?;
            self.freq = cents_to_freq_coefficient(self.parameter.value);
        }
        Ok(())
    }
}

pub struct CallbackParameter<S, G, R, T>
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

impl<S, G, R, T> CallbackParameter<S, G, R, T>
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

impl<S, G, R, T> Parameter<T> for CallbackParameter<S, G, R, T>
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
