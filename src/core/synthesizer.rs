use std::sync::{Arc, Mutex};

use super::{note::Note, oscillator::Oscillator};
use crate::{
    error::Error,
    utils::{
        evaluate::{Evaluate, EvaluateMut},
        sample_buffer::SampleBuffer,
    },
};

pub struct Synthesizer {
    notes: Arc<Mutex<Vec<Note>>>,
    osc: Oscillator,
}

impl Synthesizer {
    pub fn note_on(&mut self, note: Note) -> Result<(), Error> {
        let mut notes = self.notes.lock().expect("Cannot lock notes");
        notes.push(note);
        Ok(())
    }
}

impl EvaluateMut<Arc<Mutex<SampleBuffer>>> for Synthesizer {
    fn evaluate(&mut self, _: f32) -> Result<Arc<Mutex<SampleBuffer>>, Error> {
        let mut notes = self.notes.lock().expect("Cannot lock notes");
        for note in notes.iter_mut() {
            let t = self.osc.evaluate_note(note, 1. / 44100.)?;
            note.play_time = t;
        }
        Ok(self.osc.get_buffer())
    }
}
