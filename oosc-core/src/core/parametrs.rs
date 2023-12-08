use crate::utils::{
    convert::{exponential_time, power_to_linear, split_bipolar_pan},
    math::clamp,
};

pub trait Parametr<T>
where
    T: Clone + PartialOrd + Default,
{
    fn set_value(&mut self, value: T);
    fn get_value(&self) -> T;
    fn range(&self) -> (T, T);
}

pub struct ValueParametr<T>
where
    T: Clone,
{
    value: T,
    range: (T, T),
}

impl<T> ValueParametr<T>
where
    T: Clone + PartialOrd + Default,
{
    pub fn new(value: T, range: (T, T)) -> Self {
        Self { value, range }
    }
}

impl<T> Parametr<T> for ValueParametr<T>
where
    T: Clone + PartialOrd + Default,
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
}

pub struct OctaveParametr(ValueParametr<i32>);

impl OctaveParametr {
    pub fn new(parametr: ValueParametr<i32>) -> Self {
        Self(parametr)
    }
}

impl Parametr<i32> for OctaveParametr {
    fn set_value(&mut self, value: i32) {
        self.0.set_value(value * 12);
    }

    fn get_value(&self) -> i32 {
        self.0.get_value() / 12
    }

    fn range(&self) -> (i32, i32) {
        self.0.range()
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
}

pub struct ExponentialTimeParametr {
    pub exponential_time: f32,
    sample_rate: f32,
    linear_time: ValueParametr<f32>,
}

impl ExponentialTimeParametr {
    pub fn new(parametr: ValueParametr<f32>, sample_rate: f32) -> Self {
        Self {
            exponential_time: power_to_linear(parametr.get_value()),
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
}
