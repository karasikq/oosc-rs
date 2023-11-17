use std::sync::{Arc, Mutex};

use crate::core::note::Note;
use crate::error::Error;
use crate::utils::evaluate::{Evaluate, EvaluateMut};
use crate::utils::{adsr_envelope::ADSREnvelope, sample_buffer::SampleBuffer};

use super::time_tick::TimeTick;
use super::wavetable::WaveTable;

pub struct Oscillator {
    buffer: Arc<Mutex<SampleBuffer>>,
    envelope: ADSREnvelope,
    wavetable: Arc<Mutex<WaveTable>>,
    time: f32,
    active_notes: Vec<Note>,
}

impl TimeTick for Oscillator {
    fn tick(&mut self, delta: f32) {
        self.time += delta
    }

    fn get_time(&self) -> f32 {
        self.time
    }
}

impl EvaluateMut<Arc<Mutex<SampleBuffer>>> for Oscillator {
    fn evaluate(&mut self, t: f32) -> Result<Arc<Mutex<SampleBuffer>>, Error> {
        let mut buffer = self.buffer.lock().expect("Cannot lock buffer");
        for i in 0..buffer.len() {
            let samples = Evaluate::<(f32, f32)>::evaluate(self, t)?;
            buffer.set_at(0, i, samples.0)?;
            buffer.set_at(1, i, samples.1)?;
        }
        Ok(self.buffer.clone())
    }
}

impl Evaluate<(f32, f32)> for Oscillator {
    fn evaluate(&self, t: f32) -> Result<(f32, f32), Error> {
        let sample = self
            .wavetable
            .lock()
            .expect("Cannot lock wavetable")
            .evaluate(t)?;
        Ok((sample, sample))
    }
}
