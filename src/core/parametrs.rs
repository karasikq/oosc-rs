use std::sync::{Arc, Mutex};

use crate::error::Error;

use super::note::Converter;

pub trait Parametr<'a, T>: Send + Sync {
    fn set_value(&mut self, value: T) -> Result<&mut Self, Error>;
    fn get_value(&self) -> Result<T, Error>;
    fn range(&self) -> Result<(T, T), Error>;
}

pub struct ValueParametr<T>
where
    T: Send + Sync + Copy,
{
    value: T,
    range: (T, T),
}

impl<T> ValueParametr<T>
where
    T: Send + Sync + Copy,
{
    pub fn new(value: T, range: (T, T)) -> Self {
        Self { value, range }
    }
}

pub type SyncValueParametr<T> = Arc<Mutex<T>>;

impl<'a, T> Parametr<'a, T> for ValueParametr<T>
where
    T: Send + Sync + Copy,
{
    fn set_value(&mut self, value: T) -> Result<&mut Self, Error> {
        self.value = value;
        Ok(self)
    }

    fn get_value(&self) -> Result<T, Error> {
        Ok(self.value)
    }

    fn range(&self) -> Result<(T, T), Error> {
        Ok(self.range)
    }
}

pub struct OctaveParametr(ValueParametr<i32>);

impl OctaveParametr {
    pub fn new(parametr: ValueParametr<i32>) -> Self {
        Self(parametr)
    }
}

impl<'a> Parametr<'a, i32> for OctaveParametr {
    fn set_value(&mut self, value: i32) -> Result<&mut Self, Error> {
        self.0.set_value(value * 12)?;
        Ok(self)
    }

    fn get_value(&self) -> Result<i32, Error> {
        let value = self.0.get_value()? / 12;
        Ok(value)
    }

    fn range(&self) -> Result<(i32, i32), Error> {
        Ok(self.0.range()?)
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

impl<'a> Parametr<'a, f32> for PanParametr {
    fn set_value(&mut self, value: f32) -> Result<&mut Self, Error> {
        self.bipolar.set_value(value)?;
        self.polar = Converter::split_bipolar_pan(value);
        Ok(self)
    }

    fn get_value(&self) -> Result<f32, Error> {
        Ok(self.bipolar.get_value()?)
    }

    fn range(&self) -> Result<(f32, f32), Error> {
        Ok(self.bipolar.range()?)
    }
}
