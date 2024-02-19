use std::sync::{Arc, Mutex};

use crate::{
    core::synthesizer::Synthesizer,
    error::Error,
    midi::{mediator::MidiEventReceiver, playback::MidiPlayback},
    utils::SharedMutex,
};

use super::StreamCallback;

pub struct SynthesizerStreamCallback(pub Arc<Mutex<Synthesizer>>);

impl StreamCallback for SynthesizerStreamCallback {
    fn process_stream(
        &mut self,
        data: &mut [f32],
        _time: f32,
        _sample_rate: f32,
    ) -> std::result::Result<(), Error> {
        let mut syn = self.0.lock().unwrap();
        let buf = syn.output(data.len() / 2)?;
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
    pub SharedMutex<dyn MidiPlayback>,
    pub SharedMutex<dyn MidiEventReceiver>,
);

impl StreamCallback for MidiStreamCallback {
    fn process_stream(
        &mut self,
        _data: &mut [f32],
        time: f32,
        _sample_rate: f32,
    ) -> std::result::Result<(), Error> {
        let mut playback = self.0.lock().unwrap();
        playback.process_events(time, self.1.clone())?;
        Ok(())
    }
}
