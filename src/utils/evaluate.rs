use crate::error::Error;

pub trait Evaluate<T> {
    fn evaluate(&self, t: f32) -> Result<T, Error>;
}

pub trait EvaluateMut<T> {
    fn evaluate(&mut self, t: f32) -> Result<T, Error>;
}
