use crate::error::Error;

pub trait Evaluate<T> {
    fn evaluate(&self, t: f32) -> Result<T, Error>;
    fn evaluate_mut(&mut self, t: f32) -> Result<T, Error> {
        self.evaluate(t)
    }
}
