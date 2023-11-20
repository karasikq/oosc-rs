use std::sync::{Arc, Mutex};

use super::{note::Note, oscillator::Oscillator};
use crate::{
    error::Error,
    utils::sample_buffer::{SampleBuffer, SampleBufferBuilder, SyncSampleBuffer},
};

type Osc = Box<dyn for<'a> Oscillator<'a, &'a Note, SyncSampleBuffer>>;

pub struct Synthesizer {
    buffer: SyncSampleBuffer,
    notes: Arc<Mutex<Vec<Note>>>,
    oscillators: Vec<Osc>,
    sample_rate: f32,
    delta_time: f32,
}

impl Synthesizer {
    pub fn new(buffer_size: usize, oscillators: Vec<Osc>) -> Self {
        let buffer = SampleBufferBuilder::new()
            .set_channels(2)
            .set_samples(buffer_size)
            .build()
            .unwrap();
        Self {
            buffer: Arc::new(Mutex::new(buffer)),
            notes: Arc::new(Mutex::new(Vec::<Note>::new())),
            oscillators,
            sample_rate: 44100.0,
            delta_time: 1.0 / 44100.0,
        }
    }

    pub fn note_on(&mut self, note: Note) -> Result<(), Error> {
        let mut notes = self.notes.lock().expect("Cannot lock notes");
        notes.push(note);
        Ok(())
    }

    fn output(&mut self, delta: f32) -> Result<SyncSampleBuffer, Error> {
        let mut notes = self.notes.lock().expect("Cannot lock notes");
        let mut buffer = self.buffer.lock().expect("Cannot lock buffer");
        buffer.fill(0.);
        for osc in self.oscillators.iter_mut() {
            let buffer = osc.get_buffer();
            let mut buf = buffer.lock().expect("Cannot lock buffer");
            buf.fill(0.);
        }
        for note in notes.iter_mut() {
            for osc in self.oscillators.iter_mut() {
                osc.evaluate(delta, note)?;
            }
            note.play_time += buffer.len() as f32 * delta;
        }
        for osc in self.oscillators.iter_mut() {
            let osc_buffer = osc.get_buffer();
            let buf = osc_buffer.lock().expect("Cannot lock buffer");
            buffer.combine(&buf)?;
        }
        Ok(self.buffer.clone())
    }
}
