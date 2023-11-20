use std::sync::{Arc, Mutex};

use crate::core::note::Note;
use crate::error::Error;
use crate::utils::adsr_envelope::State;
use crate::utils::consts::PI_2M;
use crate::utils::evaluate::{Evaluate, EvaluateMut, EvaluateWithParam};
use crate::utils::{adsr_envelope::ADSREnvelope, sample_buffer::SampleBuffer};

use super::wavetable::WaveTable;

// REMINDER
// Make oscillator fully stateless, move notes to note wrapper
pub struct Oscillator {
    buffer: Arc<Mutex<SampleBuffer>>,
    envelope: ADSREnvelope,
    wavetable: WaveTable,
}

impl Oscillator {
    pub fn evaluate_note(&mut self, note: &Note, delta_time: f32) -> Result<f32, Error> {
        let mut buffer = self.buffer.lock().expect("Cannot lock buffer");
        let mut t = note.play_time;
        let mut iteration_buffer = [0.0; 2];
        for i in 0..buffer.len() {
            let envelope = match note.hold_on {
                State::None => self.envelope.evaluate(t),
                _ => {
                    if t > self.envelope.time_range_of(note.hold_on).1 {
                        self.envelope.peak_at(note.hold_on)
                    } else {
                        self.envelope.evaluate(t)
                    }
                }
            };
            let freq = PI_2M * note.frequency * t;
            let sample = self.wavetable.evaluate(freq)?;

            iteration_buffer[0] = sample * envelope * 1.;
            iteration_buffer[1] = sample * envelope * 1.;
            buffer
                .iter_buffers()
                .enumerate()
                .map(|(i, buf)| *buf.get_mut(i).unwrap() += iteration_buffer[i])
                .count();

            t += delta_time;
        }
        Ok(t)
    }

    pub fn get_buffer(&self) -> Arc<Mutex<SampleBuffer>> {
        self.buffer.clone()
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

        Ok(Oscillator {
            buffer,
            envelope,
            wavetable,
        })
    }
}
