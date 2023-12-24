use std::sync::{Mutex, Arc, RwLock};

pub mod cubic_bezier;
pub mod adsr_envelope;
pub mod evaluate;
pub mod consts;
pub mod sample_buffer;
pub mod interpolation;
pub mod math;
pub mod convert;

pub type Shared<T> = Arc<RwLock<T>>;
pub type SharedMutex<T> = Arc<Mutex<T>>;
