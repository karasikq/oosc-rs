use crate::{
    error::Error,
    utils::{
        adsr_envelope::State,
        convert::{note_to_freq, velocity_to_float},
    },
};

#[derive(Copy, Clone)]
pub struct Note {
    pub note: u32,
    pub frequency: f32,
    pub velocity: f32,
    pub play_time: f32,
    pub hold_on: State,
    pub state: State,
}

impl Note {
    pub fn new(note: u32, velocity: u32) -> Self {
        Self {
            note,
            frequency: note_to_freq(note),
            velocity: velocity_to_float(velocity),
            play_time: 0.0,
            hold_on: State::Sustain,
            state: State::Attack,
        }
    }
}

impl From<u32> for Note {
    fn from(value: u32) -> Self {
        Note::new(value, 127)
    }
}

pub trait NoteEventReceiver {
    fn note_on(&mut self, note: Note) -> Result<(), Error>;
    fn note_off(&mut self, note: u32) -> Result<(), Error>;
    fn release_all(&mut self);
}

#[cfg(test)]
mod tests {
    use crate::core::note::Note;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_note() {
        let note = Note::new(60, 127);
        assert_approx_eq!(note.frequency, 261.6, 0.05);
        let note = Note::new(69, 127);
        assert_approx_eq!(note.frequency, 440.0, 0.05);
        let note = Note::new(47, 127);
        assert_approx_eq!(note.frequency, 123.47, 0.05);
    }
}
