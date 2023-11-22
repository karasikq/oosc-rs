use std::sync::{Arc, Mutex};

use super::{note::Note, oscillator::Oscillator};
use crate::{
    error::Error,
    utils::sample_buffer::{SampleBuffer, SampleBufferBuilder, SyncSampleBuffer},
};
use rayon::prelude::*;

type Osc = Box<dyn for<'a> Oscillator<'a, &'a Note, SyncSampleBuffer>>;
type SyncNotes = Arc<Mutex<Vec<Note>>>;

pub struct Synthesizer {
    buffer: SyncSampleBuffer,
    notes: SyncNotes,
    note_buffer: SampleBuffer,
    oscillators: Vec<Osc>,
    sample_rate: u32,
    delta_time: f32,
}

#[derive(Default)]
pub struct SynthesizerBuilder {
    buffer: Option<SyncSampleBuffer>,
    note_buffer: Option<SampleBuffer>,
    oscillators: Option<Vec<Osc>>,
    sample_rate: Option<u32>,
}

impl Synthesizer {
    pub fn note_on(&mut self, note: Note) -> Result<(), Error> {
        let mut notes = self.notes.lock().expect("Cannot lock notes");
        notes.push(note);
        Ok(())
    }

    pub fn output(&mut self) -> Result<SyncSampleBuffer, Error> {
        let mut notes = self
            .notes
            .lock()
            .map_err(|e| Error::Generic(e.to_string()))?;
        let mut buffer = self
            .buffer
            .lock()
            .map_err(|e| Error::Generic(e.to_string()))?;
        let note_buffer = &mut self.note_buffer;
        buffer.fill(0.);
        for note in notes.iter_mut() {
            note_buffer.fill(0.);
            self.oscillators
                .par_iter_mut()
                .try_for_each(|osc| -> Result<(), Error> {
                    osc.evaluate(self.delta_time, note)?;
                    Ok(())
                })?;
            self.oscillators
                .iter_mut()
                .try_for_each(|osc| -> Result<(), Error> {
                    let sync_buffer = osc.get_buffer();
                    let buffer = sync_buffer
                        .lock()
                        .map_err(|e| Error::Generic(e.to_string()))?;
                    note_buffer.combine(&buffer)?;
                    Ok(())
                })?;
            note.play_time += buffer.len() as f32 * self.delta_time;
            buffer.combine(note_buffer)?;
            // Need to DCA process here
        }
        Ok(self.buffer.clone())
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
        self.buffer = Some(Arc::new(Mutex::new(
            SampleBufferBuilder::new()
                .set_channels(2)
                .set_samples(buffer_size)
                .build()?,
        )));
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

        Ok(Synthesizer {
            buffer,
            note_buffer,
            notes: Arc::new(Mutex::new(Vec::<Note>::new())),
            oscillators,
            sample_rate,
            delta_time: 1.0 / sample_rate as f32,
        })
    }
}
