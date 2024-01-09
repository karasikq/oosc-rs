use std::any::Any;

use crate::core::note::Note;
use crate::error::Error;
use crate::utils::convert::note_to_freq;
use crate::utils::evaluate::Modulation;
use crate::utils::{
    adsr_envelope::{ADSREnvelope, State},
    consts::PI_2M,
    evaluate::Evaluate,
    sample_buffer::SampleBuffer,
};
use crate::utils::{make_shared, Shared};

use super::note::NoteEventReceiver;
use super::parameter::{CallbackParameter, CentsParameter, SharedParameter, VolumeParameter};
use super::{
    parameter::{OctaveParameter, PanParameter, ValueParameter},
    wavetable::WaveTable,
};

pub trait Oscillator: Send + Sync + NoteEventReceiver {
    fn evaluate(&mut self, t: f32) -> Result<(), Error>;
    fn get_buffer_mut(&mut self) -> &mut SampleBuffer;
    fn get_buffer(&self) -> &SampleBuffer;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

struct Parametrs {
    octave_offset: Shared<OctaveParameter>,
    cents_offset: Shared<CentsParameter>,
    pan: Shared<PanParameter>,
    wt_pos: SharedParameter<i32>,
    gain: Shared<VolumeParameter>,
}

pub struct WavetableOscillator {
    buffer: SampleBuffer,
    envelope: Shared<ADSREnvelope>,
    wavetable: Shared<WaveTable>,
    notes: Vec<Note>,
    release_notes: Vec<Note>,
    parametrs: Parametrs,
    time: f32,
}

impl WavetableOscillator {
    pub fn wavetable(&self) -> Shared<WaveTable> {
        self.wavetable.clone()
    }

    pub fn envelope(&self) -> Shared<ADSREnvelope> {
        self.envelope.clone()
    }

    pub fn octave_offset(&self) -> SharedParameter<i32> {
        self.parametrs.octave_offset.clone()
    }

    pub fn cents_offset(&self) -> Shared<CentsParameter> {
        self.parametrs.cents_offset.clone()
    }

    pub fn wavetable_position(&self) -> SharedParameter<i32> {
        self.parametrs.wt_pos.clone()
    }

    pub fn pan(&self) -> Shared<PanParameter> {
        self.parametrs.pan.clone()
    }

    pub fn gain(&self) -> SharedParameter<f32> {
        self.parametrs.gain.clone()
    }

    fn get_note(&self, note: u32) -> Result<usize, Error> {
        Ok(self
            .notes
            .iter()
            .position(|x| x.note == note)
            .ok_or(format!("Note {} not playing", note))?)
    }

    fn remove_note(&mut self, index: usize) -> Note {
        self.notes.remove(index)
    }

    fn remove_released_notes(&mut self) {
        let envelope = self.envelope.read().unwrap();
        self.release_notes.retain(|note| {
            note.state != State::None && note.play_time < envelope.time_range_of(State::Release).1
        });
    }

    fn envelope_value_at(t: f32, note: &Note, adsr: Shared<ADSREnvelope>) -> f32 {
        let envelope = adsr.read().unwrap();
        match note.hold_on {
            State::None => envelope.evaluate(t),
            _ => {
                if t > envelope.time_range_of(note.hold_on).1 {
                    envelope.peak_at(note.hold_on)
                } else {
                    envelope.evaluate(t)
                }
            }
        }
    }
}

impl Oscillator for WavetableOscillator {
    fn evaluate(&mut self, delta_time: f32) -> Result<(), Error> {
        self.remove_released_notes();
        let buffer = &mut self.buffer;
        let mut pan = self.parametrs.pan.write().unwrap();
        let octave_offset = self.parametrs.octave_offset.read().unwrap().notes;
        let mut cents = self.parametrs.cents_offset.write().unwrap();
        let gain = self.parametrs.gain.read().unwrap().linear;

        (0..buffer.len()).try_for_each(|i| -> Result<(), Error> {
            let mut iteration_buffer = [0.0; 2];
            pan.next_value(delta_time)?;
            let polar_pan = pan.polar;
            cents.next_value(delta_time)?;
            let cents = cents.freq;
            self.notes
                .iter_mut()
                .chain(self.release_notes.iter_mut())
                .try_for_each(|note| -> Result<(), Error> {
                    let t = note.play_time;
                    let envelope = Self::envelope_value_at(t, note, self.envelope.clone());
                    let freq =
                        PI_2M * note_to_freq((note.note as i32 + octave_offset) as u32) * t * cents;
                    let sample = {
                        let wavetable = self.wavetable.write().unwrap();
                        wavetable.evaluate(freq)?
                    };

                    iteration_buffer[0] += sample * envelope * polar_pan.0 * note.velocity * gain;
                    iteration_buffer[1] += sample * envelope * polar_pan.1 * note.velocity * gain;

                    note.play_time += delta_time;
                    Ok(())
                })?;
            buffer
                .iter_buffers()
                .enumerate()
                .try_for_each(|(ind, buf)| -> Result<(), Error> {
                    *(buf.get_mut(i)?) = iteration_buffer[ind];
                    Ok(())
                })?;
            self.time += delta_time;
            Ok(())
        })?;
        Ok(())
    }

    fn get_buffer(&self) -> &SampleBuffer {
        &self.buffer
    }

    fn get_buffer_mut(&mut self) -> &mut SampleBuffer {
        &mut self.buffer
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NoteEventReceiver for WavetableOscillator {
    fn note_on(&mut self, note: Note) -> std::result::Result<(), Error> {
        let index = self.get_note(note.note);
        if index.is_ok() {
            self.note_off(note.note)?;
        }
        self.notes.push(note);
        Ok(())
    }

    fn note_off(&mut self, note: u32) -> std::result::Result<(), Error> {
        let index = self.get_note(note);
        let index = match index {
            Ok(i) => i,
            Err(_) => return Ok(()),
        };
        let mut note = self.remove_note(index);
        note.hold_on = State::None;
        self.release_notes.push(note);
        Ok(())
    }

    fn release_all(&mut self) {
        while let Some(note) = self.notes.pop() {
            self.release_notes.push(note);
        }
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
        let envelope = make_shared(self.envelope.take().ok_or(Error::Specify("envelope"))?);
        let wavetable = make_shared(self.wavetable.take().ok_or(Error::Specify("wavetable"))?);
        let octave_offset = make_shared(OctaveParameter::new(ValueParameter::new(0, (-2, 2))));
        let cents_offset = make_shared(CentsParameter::new(ValueParameter::new(0.0, (-100.0, 100.0))));
        let pan = make_shared(PanParameter::default());
        let gain = make_shared(VolumeParameter::default());

        let wt_clone = wavetable.clone();
        let wt_clone2 = wavetable.clone();
        let wt_clone3 = wavetable.clone();
        let wt_pos = make_shared(CallbackParameter {
            setter: move |v| {
                let _ = wt_clone.write().unwrap().set_position(v as usize);
            },
            getter: move || wt_clone2.read().unwrap().position() as i32,
            range: move || {
                let wt_range = wt_clone3.read().unwrap().position_range();
                (wt_range.0 as i32, wt_range.1 as i32)
            },
        });
        let parametrs = Parametrs {
            octave_offset,
            cents_offset,
            pan,
            wt_pos,
            gain,
        };

        Ok(WavetableOscillator {
            buffer,
            envelope,
            wavetable,
            notes: vec![],
            release_notes: vec![],
            parametrs,
            time: 0.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::{
            note::{Note, NoteEventReceiver},
            oscillator::OscillatorBuilder,
            waveshape::WaveShape,
            wavetable::WaveTableBuilder,
        },
        utils::{
            adsr_envelope::ADSREnvelope, interpolation::InterpolateMethod,
            sample_buffer::SampleBufferBuilder,
        },
    };

    #[test]
    fn test_osc_notes() {
        let adsr = ADSREnvelope::default();
        let buffer = SampleBufferBuilder::new()
            .set_channels(2)
            .set_samples(10)
            .build()
            .unwrap();
        let table = WaveTableBuilder::new()
            .from_shape(WaveShape::Sin, 10)
            .set_interpolation(InterpolateMethod::Linear)
            .build()
            .unwrap();
        let mut osc = OscillatorBuilder::new()
            .set_buffer(buffer)
            .set_envelope(adsr)
            .set_wavetable(table)
            .build()
            .unwrap();
        osc.note_on(Note::from(60)).unwrap();
        let get = osc.get_note(60);
        assert!(get.is_ok());
        let get = osc.get_note(61);
        assert!(get.is_err());
    }
}
