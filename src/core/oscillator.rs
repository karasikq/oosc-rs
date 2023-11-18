use std::sync::{Arc, Mutex};

use crate::core::note::Note;
use crate::error::Error;
use crate::utils::adsr_envelope::State;
use crate::utils::consts::PI_2M;
use crate::utils::evaluate::{Evaluate, EvaluateMut, EvaluateWithParam};
use crate::utils::{adsr_envelope::ADSREnvelope, sample_buffer::SampleBuffer};

use super::wavetable::WaveTable;

pub struct Oscillator {
    buffer: Arc<Mutex<SampleBuffer>>,
    envelope: ADSREnvelope,
    wavetable: WaveTable,
    active_notes: Arc<Mutex<Vec<Note>>>,
}

impl EvaluateMut<Arc<Mutex<SampleBuffer>>> for Oscillator {
    fn evaluate(&mut self, t: f32) -> Result<Arc<Mutex<SampleBuffer>>, Error> {
        let mut buffer = self.buffer.lock().expect("Cannot lock buffer");
        let delta_time = 1.0 / buffer.len() as f32;
        let mut time = t;
        for i in 0..buffer.len() {
            let mut notes = self.active_notes.lock().expect("Cannot lock notes");
            for note in notes.iter_mut() {
                let samples = EvaluateWithParam::evaluate(self, time, note)?;
                buffer.set_at(0, i, samples.0)?;
                buffer.set_at(1, i, samples.1)?;
                note.play_time += delta_time;
            }
            time += delta_time;
        }
        Ok(self.buffer.clone())
    }
}

impl EvaluateWithParam<&Note, (f32, f32)> for Oscillator {
    fn evaluate(&self, t: f32, note: &Note) -> Result<(f32, f32), Error> {
        let envelope = match note.hold_on {
            State::None => {
                self.envelope.evaluate(note.play_time)
            },
            _ => {
                if t > self.envelope.time_range_of(note.hold_on).1 {
                    self.envelope.peak_at(note.hold_on)
                } else {
                    self.envelope.evaluate(note.play_time)
                }
                
            }
        };
        let freq = PI_2M * note.frequency * note.play_time;
        let sample = self
            .wavetable
            /* .lock()
            .expect("Cannot lock wavetable") */
            .evaluate(freq)?;

        Ok((sample * envelope * 0.2, sample * envelope * 0.2))
    }
}
