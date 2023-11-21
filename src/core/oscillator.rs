use std::sync::{Arc, Mutex};

use crate::core::note::Note;
use crate::error::Error;
use crate::utils::adsr_envelope::State;
use crate::utils::consts::PI_2M;
use crate::utils::evaluate::Evaluate;
use crate::utils::sample_buffer::SyncSampleBuffer;
use crate::utils::{adsr_envelope::ADSREnvelope, sample_buffer::SampleBuffer};

use super::note::Converter;
use super::wavetable::WaveTable;

pub trait Oscillator<'a, T, R>: Send + Sync {
    fn evaluate(&mut self, t: f32, param: T) -> Result<R, Error>;
    fn get_buffer(&mut self) -> SyncSampleBuffer;
}

pub struct WavetableOscillator {
    buffer: SyncSampleBuffer,
    envelope: ADSREnvelope,
    wavetable: WaveTable,
    octave_offset: i32,
    pan: f32,
}

impl WavetableOscillator {
    pub fn set_octave_offset(&mut self, octave_offset: i32) -> &mut Self {
        self.octave_offset = octave_offset * 12;
        self
    }

    pub fn set_pan(&mut self, pan: f32) -> &mut Self {
        self.pan = pan;
        self
    }
}

impl Oscillator<'_, &Note, SyncSampleBuffer> for WavetableOscillator {
    fn evaluate(&mut self, delta_time: f32, note: &Note) -> Result<SyncSampleBuffer, Error> {
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
            let freq = PI_2M * Converter::note_to_freq(note.note + self.octave_offset) * t;
            let sample = self.wavetable.evaluate(freq)?;

            iteration_buffer[0] = sample * envelope * 0.2 * (1.0 - self.pan);
            iteration_buffer[1] = sample * envelope * 0.2 * self.pan;
            buffer
                .iter_buffers()
                .enumerate()
                .try_for_each(|(ind, buf)| -> Result<(), Error> {
                    *buf.get_mut(i)? += iteration_buffer[ind];
                    Ok(())
                })?;

            t += delta_time;
        }
        Ok(self.buffer.clone())
    }

    fn get_buffer(&mut self) -> SyncSampleBuffer {
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

    pub fn build(&mut self) -> Result<WavetableOscillator, Error> {
        let buffer = Arc::new(Mutex::new(
            self.buffer.take().ok_or(Error::Specify("samples buffer"))?,
        ));
        let envelope = self.envelope.take().ok_or(Error::Specify("envelope"))?;
        let wavetable = self.wavetable.take().ok_or(Error::Specify("wavetable"))?;

        Ok(WavetableOscillator {
            buffer,
            envelope,
            wavetable,
            octave_offset: 0,
            pan: 0.5,
        })
    }
}
