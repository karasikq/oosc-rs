use std::sync::{Arc, Mutex};

use crate::core::note::Note;
use crate::error::Error;
use crate::utils::adsr_envelope::State;
use crate::utils::evaluate::{Evaluate, EvaluateMut, EvaluateWithParam};
use crate::utils::{adsr_envelope::ADSREnvelope, sample_buffer::SampleBuffer};

use super::time_tick::TimeTick;
use super::wavetable::WaveTable;

pub struct Oscillator {
    buffer: Arc<Mutex<SampleBuffer>>,
    envelope: ADSREnvelope,
    wavetable: Arc<Mutex<WaveTable>>,
    time: Mutex<f32>,
    active_notes: Arc<Mutex<Vec<Note>>>,
}

impl TimeTick for Oscillator {
    fn tick(&mut self, delta: f32) -> Result<(), Error> {
        let mut time = self.time.lock().expect("Cannot lock time");
        *time += delta;
        Ok(())
    }

    fn get_time(&self) -> Result<f32, Error> {
        let time = *self.time.lock().expect("Cannot lock time");
        Ok(time)
    }
}

impl EvaluateMut<Arc<Mutex<SampleBuffer>>> for Oscillator {
    fn evaluate(&mut self, t: f32) -> Result<Arc<Mutex<SampleBuffer>>, Error> {
        let mut buffer = self.buffer.lock().expect("Cannot lock buffer");
        for i in 0..buffer.len() {
            let notes = self.active_notes.lock().expect("Cannot lock notes");
            for note in notes.iter() {
                let samples = EvaluateWithParam::evaluate(self, t, note)?;
                buffer.set_at(0, i, samples.0)?;
                buffer.set_at(1, i, samples.1)?;
            }
        }
        Ok(self.buffer.clone())
    }
}

impl EvaluateWithParam<&Note, (f32, f32)> for Oscillator {
    fn evaluate(&self, t: f32, note: &Note) -> Result<(f32, f32), Error> {
        let sample = self
            .wavetable
            .lock()
            .expect("Cannot lock wavetable")
            .evaluate(t)?;
        /* let envelope = match note.hold_state() {
            State::None => {
                self.envelope.evaluate(t)
            },
            state => {
                
            }
        }; */
        Ok((sample, sample))
    }
}
