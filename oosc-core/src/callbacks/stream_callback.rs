use std::sync::{Arc, Mutex};

use crate::{
    core::synthesizer::Synthesizer,
    error::Error,
    midi::{
        mediator::{MidiEventReceiver, MidiSynthesizerMediator},
        playback::{MidiPlayback, PlaybackState},
    },
};

use super::StreamCallback;

pub struct SynthesizerStreamCallback(pub Arc<Mutex<Synthesizer>>);

impl StreamCallback for SynthesizerStreamCallback {
    fn process_stream(&mut self, data: &mut [f32], _time: f32) -> std::result::Result<(), Error> {
        let mut syn = self.0.lock().unwrap();
        let buf = syn.output()?;
        let mut l = buf.iter(0)?;
        let mut r = buf.iter(1)?;
        for frame in data.chunks_exact_mut(2) {
            frame[0] = l.next().ok_or("Cannot get next sample")?;
            frame[1] = r.next().ok_or("Cannot get next sample")?;
        }

        Ok(())
    }
}

pub struct MidiStreamCallback(
    pub Arc<Mutex<dyn MidiPlayback>>,
    pub Arc<Mutex<Synthesizer>>,
);

impl StreamCallback for MidiStreamCallback {
    fn process_stream(&mut self, _data: &mut [f32], time: f32) -> std::result::Result<(), Error> {
        let mut playback = self.0.lock().unwrap();
        if let PlaybackState::None = playback.get_state() {
            return Ok(());
        }
        let syn = self.1.clone();
        let mut synth_mediator: Box<dyn MidiEventReceiver> =
            Box::new(MidiSynthesizerMediator::new(syn));
        playback.process_events(time, &mut synth_mediator)?;
        Ok(())
    }
}

