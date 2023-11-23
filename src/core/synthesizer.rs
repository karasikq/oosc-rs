use rayon::prelude::*;

use super::{
    amplifier::Amplifier,
    note::{Converter, Note},
    oscillator::Oscillator,
};
use crate::{
    error::Error,
    utils::sample_buffer::{SampleBuffer, SampleBufferBuilder},
};
use rayon::prelude::*;

type Osc = Box<dyn for<'a> Oscillator<'a, &'a Note, ()>>;

pub struct Synthesizer {
    buffer: SampleBuffer,
    notes: Vec<Note>,
    note_buffer: SampleBuffer,
    oscillators: Vec<Osc>,
    amplifier: Amplifier,
    sample_rate: u32,
    delta_time: f32,
}

#[derive(Default)]
pub struct SynthesizerBuilder {
    buffer: Option<SampleBuffer>,
    note_buffer: Option<SampleBuffer>,
    oscillators: Option<Vec<Osc>>,
    sample_rate: Option<u32>,
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
        self.amplifier.process(buffer)?;
        Ok(&self.buffer)
    }
}

impl SynthesizerBuilder {
    pub fn new() -> Self {
        Self {
            buffer: None,
            note_buffer: None,
            oscillators: None,
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
        let dca = Amplifier::new(
            Converter::voltage_to_linear(-3.0),
            Converter::split_bipolar_pan(0.0),
        );
        Ok(Synthesizer {
            buffer,
            note_buffer,
            notes: Vec::<Note>::new(),
            oscillators,
            amplifier: dca,
            sample_rate,
            delta_time: 1.0 / sample_rate as f32,
        })
    }
}
