use crate::effects::amplifier::Amplifier;
use crate::effects::effect::Effect;
use rayon::prelude::*;

use super::{
    note::Note,
    oscillator::Oscillator,
    parametrs::{PanParametr, ValueParametr, VolumeParametr},
};
use crate::{
    error::Error,
    utils::sample_buffer::{SampleBuffer, SampleBufferBuilder},
};

type Osc = Box<dyn for<'a> Oscillator<'a, &'a Note, ()> + Sync + Send>;
type SynEffect = Box<dyn for<'a> Effect<'a> + Sync + Send>;

pub struct Synthesizer {
    buffer: SampleBuffer,
    notes: Vec<Note>,
    note_buffer: SampleBuffer,
    oscillators: Vec<Osc>,
    effects: Vec<SynEffect>,
    sample_rate: u32,
    delta_time: f32,
}

impl Synthesizer {
    pub fn note_on(&mut self, note: Note) -> Result<(), Error> {
        self.notes.push(note);
        Ok(())
    }

    pub fn output(&mut self) -> Result<&SampleBuffer, Error> {
        let buffer = &mut self.buffer;
        let note_buffer = &mut self.note_buffer;
        buffer.fill(0.);
        for note in self.notes.iter_mut() {
            note_buffer.fill(0.);
            self.oscillators
                .par_iter_mut()
                .try_for_each(|osc| -> Result<(), Error> { osc.evaluate(self.delta_time, note) })?;
            self.oscillators
                .iter_mut()
                .try_for_each(|osc| -> Result<(), Error> {
                    let buffer = osc.get_buffer();
                    note_buffer.combine(buffer)?;
                    Ok(())
                })?;
            note.play_time += buffer.len() as f32 * self.delta_time;
            buffer.combine(note_buffer)?;
        }
        self.effects
            .iter()
            .try_for_each(|effect| -> Result<(), Error> { effect.process(buffer) })?;
        Ok(&self.buffer)
    }

    pub fn get_buffer(&self) -> &SampleBuffer {
        &self.buffer
    }
}

#[derive(Default)]
pub struct SynthesizerBuilder {
    buffer: Option<SampleBuffer>,
    note_buffer: Option<SampleBuffer>,
    oscillators: Option<Vec<Osc>>,
    effects: Option<Vec<SynEffect>>,
    sample_rate: Option<u32>,
}

impl SynthesizerBuilder {
    pub fn new() -> Self {
        Self {
            buffer: None,
            note_buffer: None,
            oscillators: None,
            effects: None,
            sample_rate: None,
        }
    }

    pub fn set_buffer(&mut self, buffer_size: usize) -> Result<&mut Self, Error> {
        self.buffer = Some(
            SampleBufferBuilder::new()
                .set_channels(2)
                .set_samples(buffer_size)
                .build()?,
        );
        self.note_buffer = Some(
            SampleBufferBuilder::new()
                .set_channels(2)
                .set_samples(buffer_size)
                .build()?,
        );
        Ok(self)
    }

    pub fn add_osc(&mut self, osc: Osc) -> &mut Self {
        if let Some(vec) = self.oscillators.as_mut() {
            vec.push(osc);
        } else {
            self.oscillators = Some(vec![osc]);
        }
        self
    }

    pub fn add_effect(&mut self, effect: SynEffect) -> &mut Self {
        if let Some(effects) = self.effects.as_mut() {
            effects.push(effect);
        } else {
            self.effects = Some(vec![effect]);
        }
        self
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) -> &mut Self {
        self.sample_rate = Some(sample_rate);
        self
    }

    pub fn build(&mut self) -> Result<Synthesizer, Error> {
        let buffer = self.buffer.take().ok_or(Error::Specify("buffer size"))?;
        let note_buffer = self
            .note_buffer
            .take()
            .ok_or(Error::Specify("buffer size"))?;
        let oscillators = self
            .oscillators
            .take()
            .ok_or(Error::Specify("oscillators"))?;
        let sample_rate = self.sample_rate.ok_or(Error::Specify("sample_rate"))?;
        let amplifier = Amplifier::new(
            VolumeParametr::from(ValueParametr::new(-3.0, (-60.0, 3.0))),
            PanParametr::from(ValueParametr::new(0.0, (-1.0, 1.0))),
        );
        self.add_effect(Box::new(amplifier));
        let effects = self.effects.take().unwrap();

        Ok(Synthesizer {
            buffer,
            note_buffer,
            notes: Vec::<Note>::new(),
            oscillators,
            effects,
            sample_rate,
            delta_time: 1.0 / sample_rate as f32,
        })
    }
}
