use midly::TrackEvent;

use crate::{
    core::{
        note::Note,
        synthesizer::{SyncSynthesizer, Synthesizer},
    },
    error::Error,
};

pub trait MidiEventReceiver {
    fn receive_event(&mut self, event: &TrackEvent) -> Result<(), Error>;
}

pub struct MidiSynthesizerMediator {
    synthesizer: SyncSynthesizer,
}

impl MidiSynthesizerMediator {
    pub fn new(synthesizer: SyncSynthesizer) -> Self {
        Self { synthesizer }
    }
}

impl MidiEventReceiver for MidiSynthesizerMediator {
    fn receive_event(&mut self, event: &TrackEvent) -> Result<(), Error> {
        let mut syn = self.synthesizer.lock().unwrap();
        match event.kind {
            midly::TrackEventKind::Midi { message, .. } => match message {
                midly::MidiMessage::NoteOn { key, vel } => {
                    let key = key.as_int();
                    let vel = vel.as_int();
                    syn.note_on(Note::new(key.into(), vel.into()))?;
                }
                midly::MidiMessage::NoteOff { key, .. } => {
                    let key = key.as_int();
                    syn.note_off(key.into())?;
                }
                _ => (),
            },
            midly::TrackEventKind::SysEx(_) => (),
            midly::TrackEventKind::Escape(_) => (),
            midly::TrackEventKind::Meta(_) => (),
        };
        Ok(())
    }
}
