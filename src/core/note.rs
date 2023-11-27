use crate::{utils::{adsr_envelope::State, consts::PI_4}, error::Error};

#[derive(Copy, Clone)]
pub struct Note {
    pub note: i32,
    pub frequency: f32,
    pub velocity: f32,
    pub play_time: f32,
    pub hold_on: State,
    pub state: State,
}

impl Note {
    pub fn new(note: i32, velocity: i32) -> Self {
        Self {
            note,
            frequency: Converter::note_to_freq(note),
            velocity: Converter::velocity_to_float(velocity),
            play_time: 0.0,
            hold_on: State::Sustain,
            state: State::Attack,
        }
    }
}

impl From<i32> for Note {
    fn from(value: i32) -> Self {
        Note::new(value, 127)
    }
}

pub trait NoteEventReceiver {
    fn note_on(&mut self, note: Note) -> Result<(), Error>;
    fn note_off(&mut self, note: i32) -> Result<(), Error>;
}

pub struct Converter;

impl Converter {
    #[inline]
    pub fn cents_to_freq(cents: i32) -> f32 {
        2.0_f32.powf(cents as f32 / 1200.0)
    }

    #[inline]
    pub fn velocity_to_float(velocity: i32) -> f32 {
        let velocity_f32 = velocity as f32;
        velocity_f32 * velocity_f32 / (127.0 * 127.0)
    }

    #[inline]
    pub fn note_to_freq(note: i32) -> f32 {
        8.175_799_f32 * 1.059_463_1_f32.powi(note)
    }

    #[inline]
    pub fn linear_to_power(value: f32) -> f32 {
        10. * value.log10()
    }

    #[inline]
    pub fn power_to_linear(value: f32) -> f32 {
        10.0_f32.powf(value / 10.0)
    }

    #[inline]
    pub fn voltage_to_linear(value: f32) -> f32 {
        10.0_f32.powf(value / 20.0)
    }

    #[inline]
    pub fn split_bipolar_pan(value: f32) -> (f32, f32) {
        // Const-power pan
        // Use tables for cos/sin ?
        (
            (PI_4 * (value + 1.0)).cos(),
            (PI_4 * (value + 1.0)).sin(),
        )
    }
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
