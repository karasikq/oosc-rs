use crate::error::Error;

pub trait TimeTick {
    fn tick(&mut self, delta: f32) -> Result<(), Error>;
    fn get_time(&self) -> Result<f32, Error>;
}
