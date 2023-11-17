use crate::utils::adsr_envelope::State;

pub struct Note {
    note: u8,
    frequency: f32,
    velocity: f32,
    play_time: f32,
    hold_on: State,
}

impl Note {
    pub fn new(note: u8, velocity: u8) -> Self {
        Self {
            note,
            frequency: Converter::note_to_freq(note),
            velocity: Converter::velocity_to_float(velocity),
            play_time: 0.0,
            hold_on: State::Sustain,
        }
    }

}

pub struct Converter;

impl Converter {
    #[inline]
    pub fn cents_to_freq(cents: u8) -> f32 {
        2.0_f32.powf(cents as f32 / 1200.0)
    }

    #[inline]
    fn velocity_to_float(velocity: u8) -> f32 {
        velocity as f32 / 255.0
    }

    #[inline]
    fn note_to_freq(note: u8) -> f32 {
        8.175_799_f32 * 1.059_463_1_f32.powi(note as i32)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::note::Note;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_note() {
        let note = Note::new(60, 255);
        assert_approx_eq!(note.frequency, 261.6, 0.05);
        let note = Note::new(69, 255);
        assert_approx_eq!(note.frequency, 440.0, 0.05);
        let note = Note::new(47, 255);
        assert_approx_eq!(note.frequency, 123.47, 0.05);
    }
}