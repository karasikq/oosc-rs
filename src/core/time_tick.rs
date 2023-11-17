pub trait TimeTick {
    fn tick(&mut self, delta: f32);
    fn get_time(&self) -> f32;
}
