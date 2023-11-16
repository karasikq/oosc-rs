use crate::error::Error;

pub trait Evaluate {
    fn evaluate(&self, t: f32) -> Result<f32, Error>;
}
