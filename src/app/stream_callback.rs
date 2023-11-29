use std::sync::{Arc, Mutex};

use midly::Smf;

use crate::{
    core::synthesizer::Synthesizer,
    error::Error,
    midi::{
        mediator::{MidiEventReceiver, MidiSynthesizerMediator},
        playback::{BoxedMidiPlayback, OptionalPlayback, PlaybackControl, SmfPlayback},
    },
};

pub trait StreamCallback: Send + Sync {
    fn process_stream(&mut self, data: &mut [f32], time: f32) -> Result<(), Error>;
}

pub struct SynthesizerStreamCallback(pub Arc<Mutex<Synthesizer>>);
pub struct MidiStreamCallback(
    pub Arc<Mutex<BoxedMidiPlayback>>,
    pub Arc<Mutex<Synthesizer>>,
);

impl StreamCallback for SynthesizerStreamCallback {
    fn process_stream(&mut self, data: &mut [f32], _time: f32) -> std::result::Result<(), Error> {
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

impl StreamCallback for MidiStreamCallback {
    fn process_stream(&mut self, _data: &mut [f32], time: f32) -> std::result::Result<(), Error> {
        let mut playback = self.0.lock().unwrap();
        if playback.is_none() {
            return Ok(());
        }
        let playback = playback.as_mut().unwrap();
        let syn = self.1.clone();
        let mut synth_mediator: Box<dyn MidiEventReceiver> =
            Box::new(MidiSynthesizerMediator::new(syn));
        playback.play(time, &mut synth_mediator).unwrap();
        Ok(())
    }
}