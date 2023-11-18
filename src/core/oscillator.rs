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
    notes: Arc<Mutex<Vec<Note>>>,
    envelope: ADSREnvelope,
    wavetable: WaveTable,
}

impl EvaluateMut<Arc<Mutex<SampleBuffer>>> for Oscillator {
    fn evaluate(&mut self, t: f32) -> Result<Arc<Mutex<SampleBuffer>>, Error> {
        let mut buffer = self.buffer.lock().expect("Cannot lock buffer");
        let delta_time = 1.0 / buffer.len() as f32;
        let mut time = t;
        for i in 0..buffer.len() {
            let mut notes = self.notes.lock().expect("Cannot lock notes");
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
            State::None => self.envelope.evaluate(note.play_time),
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

#[derive(Default)]
pub struct OscillatorBuilder {
    buffer: Option<SampleBuffer>,
    envelope: Option<ADSREnvelope>,
    wavetable: Option<WaveTable>,
}

impl OscillatorBuilder {
    pub fn new() -> Self {
        Self {
            buffer: None,
            envelope: None,
            wavetable: None,
        }
    }

    pub fn set_buffer(&mut self, buffer: SampleBuffer) -> &mut Self {
        self.buffer = Some(buffer);
        self
    }

    pub fn set_envelope(&mut self, envelope: ADSREnvelope) -> &mut Self {
        self.envelope = Some(envelope);
        self
    }

    pub fn set_wavetable(&mut self, wavetable: WaveTable) -> &mut Self {
        self.wavetable = Some(wavetable);
        self
    }

    pub fn build(&mut self) -> Result<Oscillator, Error> {
        let buffer = Arc::new(Mutex::new(
            self.buffer.take().ok_or(Error::Specify("samples buffer"))?,
        ));
        let envelope = self.envelope.take().ok_or(Error::Specify("envelope"))?;
        let wavetable = self.wavetable.take().ok_or(Error::Specify("wavetable"))?;
        let notes = Arc::new(Mutex::new(Vec::<Note>::new()));

        Ok(Oscillator {
            buffer,
            notes,
            envelope,
            wavetable,
        })
    }
}
