use midly::TrackEvent;

use crate::{
    core::{note::Note, synthesizer::Synthesizer},
    error::Error,
};

pub trait MidiEventReceiver<'a> {
    fn receive_event(&mut self, event: &TrackEvent) -> Result<(), Error>;
}

pub struct MidiSynthesizerMediator<'a> {
    synthesizer: &'a mut Synthesizer,
}

impl<'a> MidiSynthesizerMediator<'a> {
    pub fn new(synthesizer: &'a mut Synthesizer) -> Self {
        Self { synthesizer }
    }
}

impl<'a> MidiEventReceiver<'a> for MidiSynthesizerMediator<'a> {
    fn receive_event(&mut self, event: &TrackEvent) -> Result<(), Error> {
        match event.kind {
            midly::TrackEventKind::Midi { message, .. } => match message {
                midly::MidiMessage::NoteOn { key, vel } => {
                    let key = key.as_int();
                    let vel = vel.as_int();
                    self.synthesizer
                        .note_on(Note::new(key.into(), vel.into()))?;
                }
                midly::MidiMessage::NoteOff { key, .. } => {
                    let key = key.as_int();
                    self.synthesizer.note_off(key.into())?;
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
