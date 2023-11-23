use crate::core::note::Converter;
use crate::core::note::Note;
use crate::error::Error;
use crate::utils::adsr_envelope::State;
use crate::utils::consts::PI_2M;
use crate::utils::evaluate::Evaluate;
use crate::utils::{adsr_envelope::ADSREnvelope, sample_buffer::SampleBuffer};

use super::parametrs::OctaveParametr;
use super::parametrs::PanParametr;
use super::parametrs::Parametr;
use super::parametrs::ValueParametr;
use super::wavetable::WaveTable;

pub trait Oscillator<'a, T, R>: Send + Sync {
    fn evaluate(&mut self, t: f32, param: T) -> Result<R, Error>;
    fn get_buffer(&mut self) -> &mut SampleBuffer;
}

pub struct WavetableOscillator {
    buffer: SampleBuffer,
    envelope: ADSREnvelope,
    wavetable: WaveTable,
    octave_offset: OctaveParametr,
    pan: PanParametr,
}

impl WavetableOscillator {
    pub fn get_envelope(&mut self) -> &mut ADSREnvelope {
        &mut self.envelope
    }

    pub fn get_octave_offset(&mut self) -> &mut impl Parametr<i32> {
        &mut self.octave_offset
    }

    pub fn get_pan(&mut self) -> &mut impl Parametr<f32> {
        &mut self.pan
    }
}

impl Oscillator<'_, &Note, ()> for WavetableOscillator {
    fn evaluate(&mut self, delta_time: f32, note: &Note) -> Result<(), Error> {
        let buffer = &mut self.buffer;
        let pan = &self.pan;
        let octave_offset = &self.octave_offset;
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
            let freq = PI_2M * Converter::note_to_freq(note.note + octave_offset.get_value()?) * t;
            let sample = self.wavetable.evaluate(freq)?;

            iteration_buffer[0] = sample * envelope * pan.polar.0 * note.velocity;
            iteration_buffer[1] = sample * envelope * pan.polar.1 * note.velocity;
            buffer
                .iter_buffers()
                .enumerate()
                .try_for_each(|(ind, buf)| -> Result<(), Error> {
                    buf.set_at(i, iteration_buffer[ind])?;
                    Ok(())
                })?;

            t += delta_time;
        }
        Ok(())
    }

    fn get_buffer(&mut self) -> &mut SampleBuffer {
        &mut self.buffer
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
        let buffer = self.buffer.take().ok_or(Error::Specify("samples buffer"))?;
        let envelope = self.envelope.take().ok_or(Error::Specify("envelope"))?;
        let wavetable = self.wavetable.take().ok_or(Error::Specify("wavetable"))?;
        let octave_offset = OctaveParametr::new(ValueParametr::new(0, (-2, 2)));
        let pan = PanParametr::new(ValueParametr::new(0., (-1., 1.)));

        Ok(WavetableOscillator {
            buffer,
            envelope,
            wavetable,
            octave_offset,
            pan,
        })
    }
}
