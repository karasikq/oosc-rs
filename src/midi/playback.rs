use midly::{Smf, Timing};

use crate::error::Error;

use super::mediator::MidiEventReceiver;

pub struct Playback<'a> {
    midi_ticks: u32,
    tps: f32,
    bpm: f32,
    ppq: u32,
    data: Smf<'a>,
}

impl<'a> Playback<'a> {
    pub fn new(data: Smf<'a>) -> Self {
        let ppq = match data.header.timing {
            Timing::Metrical(v) => v.into(),
            Timing::Timecode(_, _) => 192,
        };
        let bpm = 120.0;
        let tps = Self::calculate_tps(bpm, ppq.into());
        Self {
            midi_ticks: 0,
            tps,
            bpm,
            ppq: ppq.into(),
            data,
        }
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm;
        self.tps = Self::calculate_tps(self.bpm, self.ppq);
    }

    pub fn play<T>(&mut self, t: f32, event_receiver: &'a mut T) -> Result<(), Error>
    where
        T: MidiEventReceiver<'a>,
    {
        let playback_time_ticks: u32 = (t * self.tps) as u32;
        let playback_midi_ticks: u32 = self.midi_ticks;
        let delta_ticks = playback_time_ticks - playback_midi_ticks;
        let mut last_event_ticks: u32 = playback_midi_ticks;
        let mut current_ticks: u32 = 0;

        for track in self.data.tracks.iter().skip(1) {
            for event in track.iter() {
                current_ticks += event.delta.as_int();
                if current_ticks > playback_midi_ticks + delta_ticks {
                    self.midi_ticks = match last_event_ticks {
                        0 => 1,
                        v => v,
                    };
                    return Ok(());
                }
                if current_ticks > playback_midi_ticks
                    || (current_ticks == 0 && playback_midi_ticks == 0)
                {
                    event_receiver.receive_event(event)?;
                    last_event_ticks = current_ticks;
                }
            }
        }

        self.midi_ticks = match last_event_ticks {
            0 => 1,
            v => v,
        };
        Ok(())
    }

    fn reset(&mut self) {
        self.midi_ticks = 0;
    }

    fn calculate_tps(bpm: f32, ppq: u32) -> f32 {
        (bpm * ppq as f32) / 60.
    }
}
