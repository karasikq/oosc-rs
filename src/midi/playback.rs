use midly::{Smf, Timing};

use crate::error::Error;

use super::mediator::MidiEventReceiver;

pub trait PlaybackControl: Sync + Send {
    // fn load<'a>(&mut self, data: Smf<'a>);
    fn set_bpm(&mut self, bpm: f32);
    fn play<'b>(
        &mut self,
        delta_time: f32,
        event_receiver: &mut Box<dyn MidiEventReceiver>,
    ) -> Result<(), Error>;
    fn reset(&mut self);
}

pub type BoxedMidiPlayback = Option<Box<dyn PlaybackControl>>;

pub struct SmfPlayback<'a> {
    time: f32,
    midi_ticks: u32,
    tps: f32,
    bpm: f32,
    ppq: u32,
    data: Smf<'a>,
}

pub type OptionalPlayback<'a> = Option<SmfPlayback<'a>>;

impl<'a> SmfPlayback<'a> {
    pub fn new(data: Smf<'a>) -> Self {
        let ppq = match data.header.timing {
            Timing::Metrical(v) => v.into(),
            Timing::Timecode(_, _) => 192,
        };
        let bpm = 120.0;
        let tps = Self::calculate_tps(bpm, ppq.into());
        Self {
            time: 0.0,
            midi_ticks: 0,
            tps,
            bpm,
            ppq: ppq.into(),
            data,
        }
    }

    fn calculate_tps(bpm: f32, ppq: u32) -> f32 {
        (bpm * ppq as f32) / 60.
    }
}

impl<'a> PlaybackControl for SmfPlayback<'a> {
    fn play<'b>(
        &mut self,
        delta_time: f32,
        event_receiver: &mut Box<dyn MidiEventReceiver>,
    ) -> Result<(), Error> {
        let t = self.time + delta_time;
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
                    let receiver = event_receiver.as_mut();
                    receiver.receive_event(event)?;
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

    fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm;
        self.tps = Self::calculate_tps(self.bpm, self.ppq);
    }

    fn reset(&mut self) {
        self.time = 0.0;
        self.midi_ticks = 0;
    }

    /* fn load<'b>(&mut self, data: Smf<'b>) {
        self.data = data;
        self.ppq = match data.header.timing {
            Timing::Metrical(v) => v.into() as u32,
            Timing::Timecode(_, _) => 192,
        };
        self.bpm = 120.0;
        self.tps = Self::calculate_tps(bpm, self.ppq);
    } */
}
