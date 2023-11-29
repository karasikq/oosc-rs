use std::sync::{Mutex, Arc};

use crate::{
    core::synthesizer::Synthesizer,
    error::Error,
    midi::{mediator::MidiSynthesizerMediator, playback::Playback},
};

pub trait StreamCallback: Send + Sync {
    fn process_stream(&mut self, data: &mut [f32], time: f32) -> Result<(), Error>;
}

pub struct SynthesizerStreamCallback(pub Arc<Mutex<Synthesizer>>);
pub struct MidiStreamCallback<'a>(&'a mut Playback<'a>, &'a mut Synthesizer);

impl<'a> StreamCallback for SynthesizerStreamCallback {
    fn process_stream(&mut self, data: &mut [f32], time: f32) -> std::result::Result<(), Error> {
        let mut syn = self.0.lock().unwrap();
        let buf = syn.output()?;
        let mut b = buf.iter(0)?;
        for frame in data.chunks_exact_mut(2) {
            let s = b.next().ok_or("Cannot get next sample")?;
            for f in frame.iter_mut() {
                *f = s;
            }
        }

        Ok(())
    }
}

impl<'a> StreamCallback for MidiStreamCallback<'a> {
    fn process_stream(&mut self, _data: &mut [f32], time: f32) -> std::result::Result<(), Error> {
        let mut synth_mediator = MidiSynthesizerMediator::new(self.1);
        // self.0.play(time, synth_mediator).unwrap();

        Ok(())
    }
}
