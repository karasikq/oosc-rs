use std::sync::{Arc, Mutex};

use super::{note::Note, oscillator::Oscillator};
use crate::{
    error::Error,
    utils::sample_buffer::{SampleBufferBuilder, SyncSampleBuffer},
};
use rayon::prelude::*;

type Osc = Box<dyn for<'a> Oscillator<'a, &'a Note, SyncSampleBuffer>>;
type SyncNotes = Arc<Mutex<Vec<Note>>>;

pub struct Synthesizer {
    buffer: SyncSampleBuffer,
    notes: SyncNotes,
    oscillators: Vec<Osc>,
    sample_rate: u32,
    delta_time: f32,
}

#[derive(Default)]
pub struct SynthesizerBuilder {
    buffer: Option<SyncSampleBuffer>,
    oscillators: Option<Vec<Osc>>,
    sample_rate: Option<u32>,
}

impl Synthesizer {
    pub fn note_on(&mut self, note: Note) -> Result<(), Error> {
        let mut notes = self.notes.lock().expect("Cannot lock notes");
        notes.push(note);
        Ok(())
    }

    fn output(&mut self) -> Result<SyncSampleBuffer, Error> {
        let mut notes = self.notes.lock().expect("Cannot lock notes");
        let mut buffer = self.buffer.lock().expect("Cannot lock buffer");
        buffer.fill(0.);
        for osc in self.oscillators.iter_mut() {
            let buffer = osc.get_buffer();
            let mut buf = buffer.lock().expect("Cannot lock buffer");
            buf.fill(0.);
        }
        for note in notes.iter_mut() {
            self.oscillators.par_iter_mut().try_for_each(|osc| -> Result<(), Error> {
                osc.evaluate(self.delta_time, note)?;
                Ok(())
            })?;
            note.play_time += buffer.len() as f32 * self.delta_time;
        }
        for osc in self.oscillators.iter_mut() {
            let osc_buffer = osc.get_buffer();
            let buf = osc_buffer.lock().expect("Cannot lock buffer");
            buffer.combine(&buf)?;
        }
        Ok(self.buffer.clone())
    }
}

impl SynthesizerBuilder {
    pub fn new() -> Self {
        Self {
            buffer: None,
            oscillators: None,
            sample_rate: None,
        }
    }

    pub fn set_buffer(&mut self, buffer_size: usize) -> Result<&mut Self, Error> {
        self.buffer = Some(Arc::new(Mutex::new(
            SampleBufferBuilder::new()
                .set_channels(2)
                .set_samples(buffer_size)
                .build()?,
        )));
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
        let oscillators = self
            .oscillators
            .take()
            .ok_or(Error::Specify("oscillators"))?;
        let sample_rate = self.sample_rate.ok_or(Error::Specify("sample_rate"))?;

        Ok(Synthesizer {
            buffer,
            notes: Arc::new(Mutex::new(Vec::<Note>::new())),
            oscillators,
            sample_rate,
            delta_time: 1.0 / sample_rate as f32,
        })
    }
}
