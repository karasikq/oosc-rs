use super::note::Converter;
use crate::{error::Error, utils::math::clamp};

pub trait Parametr<T>
where
    T: Clone + PartialOrd + Default,
{
    fn set_value(&mut self, value: T) -> Result<(), Error>;
    fn get_value(&self) -> Result<T, Error>;
    fn get_value_or_default(&self) -> T {
        let value = self.get_value().ok();
        match value {
            Some(v) => v,
            None => T::default(),
        }
    }
    fn range(&self) -> Result<(T, T), Error>;
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
    fn set_value(&mut self, value: T) -> Result<(), Error> {
        self.value = clamp(value, &self.range);
        Ok(())
    }

    fn get_value(&self) -> Result<T, Error> {
        Ok(self.value.clone())
    }

    fn range(&self) -> Result<(T, T), Error> {
        Ok(self.range.clone())
    }
}

pub struct OctaveParametr(ValueParametr<i32>);

impl OctaveParametr {
    pub fn new(parametr: ValueParametr<i32>) -> Self {
        Self(parametr)
    }
}

impl Parametr<i32> for OctaveParametr {
    fn set_value(&mut self, value: i32) -> Result<(), Error> {
        self.0.set_value(value * 12)?;
        Ok(())
    }

    fn get_value(&self) -> Result<i32, Error> {
        let value = self.0.get_value()? / 12;
        Ok(value)
    }

    fn range(&self) -> Result<(i32, i32), Error> {
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
            polar: Converter::split_bipolar_pan(parametr.get_value().unwrap()),
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
    fn set_value(&mut self, value: f32) -> Result<(), Error> {
        self.bipolar.set_value(value)?;
        self.polar = Converter::split_bipolar_pan(value);
        Ok(())
    }

    fn get_value(&self) -> Result<f32, Error> {
        self.bipolar.get_value()
    }

    fn range(&self) -> Result<(f32, f32), Error> {
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
            linear: Converter::power_to_linear(parametr.get_value().unwrap()),
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
    fn set_value(&mut self, value: f32) -> Result<(), Error> {
        self.db.set_value(value)?;
        self.linear = Converter::power_to_linear(value);
        Ok(())
    }

    fn get_value(&self) -> Result<f32, Error> {
        self.db.get_value()
    }

    fn range(&self) -> Result<(f32, f32), Error> {
        self.db.range()
    }
}
