use crate::{
    core::{note::Note, synthesizer::SyncSynthesizer},
    error::Error,
};

use super::smf_extensions::{OwnedTrackEvent, OwnedTrackEventKind};

pub trait MidiEventReceiver: Send + Sync {
    fn receive_event(&mut self, event: &OwnedTrackEvent) -> Result<(), Error>;
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
    fn receive_event(&mut self, event: &OwnedTrackEvent) -> Result<(), Error> {
        let mut syn = self.synthesizer.lock().unwrap();
        match event.kind {
            OwnedTrackEventKind::Midi { message, .. } => match message {
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
            OwnedTrackEventKind::SysEx(_) => (),
            OwnedTrackEventKind::Escape(_) => (),
            OwnedTrackEventKind::Meta(_) => (),
        };
        Ok(())
    }
}
