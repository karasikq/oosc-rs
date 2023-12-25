use std::sync::{Arc, Mutex, RwLock};

pub mod adsr_envelope;
pub mod consts;
pub mod convert;
pub mod cubic_bezier;
pub mod evaluate;
pub mod interpolation;
pub mod math;
pub mod sample_buffer;

pub type Shared<T> = Arc<RwLock<T>>;
pub type SharedMutex<T> = Arc<Mutex<T>>;

pub fn make_shared<T>(value: T) -> Shared<T> {
    Arc::new(RwLock::new(value))
}

pub fn make_shared_mutex<T>(value: T) -> SharedMutex<T> {
    Arc::new(Mutex::new(value))
}
