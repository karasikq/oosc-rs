use midly::{Smf, Timing};

use crate::{error::Error, utils::SharedMutex};

use super::{mediator::MidiEventReceiver, smf_extensions::OwnedSmf};

#[derive(Clone, Copy)]
pub enum PlaybackState {
    None,
    Playing(f32),
    Paused(f32),
    Stoped,
}

pub trait MidiPlayback: Sync + Send {
    fn load(&mut self, data: Smf<'_>);
    fn play(&mut self);
    fn set_bpm(&mut self, bpm: f32);
    fn get_state(&self) -> PlaybackState;
    fn process_events(
        &mut self,
        delta_time: f32,
        event_receiver: SharedMutex<dyn MidiEventReceiver>,
    ) -> Result<(), Error>;
    fn reset(&mut self);
}

pub struct SmfPlayback {
    time: f32,
    midi_ticks: u32,
    tps: f32,
    bpm: f32,
    ppq: u32,
    data: Option<OwnedSmf>,
    state: PlaybackState,
}

impl SmfPlayback {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            midi_ticks: 0,
            tps: 0.0,
            bpm: 0.0,
            ppq: 0,
            data: None,
            state: PlaybackState::None,
        }
    }

    pub fn from_smf(data: Smf<'_>) -> Result<Self, Error> {
        let data = OwnedSmf::try_from(&data)?;
        let ppq = match data.header.timing {
            Timing::Metrical(v) => v.into(),
            Timing::Timecode(_, _) => 192,
        };
        let bpm = 120.0;
        let tps = Self::calculate_tps(bpm, ppq.into());
        Ok(Self {
            time: 0.0,
            midi_ticks: 0,
            tps,
            bpm,
            ppq: ppq.into(),
            data: Some(data),
            state: PlaybackState::Stoped,
        })
    }

    fn calculate_tps(bpm: f32, ppq: u32) -> f32 {
        (bpm * ppq as f32) / 60.
    }
}

impl Default for SmfPlayback {
    fn default() -> Self {
        Self::new()
    }
}

impl MidiPlayback for SmfPlayback {
    fn process_events(
        &mut self,
        delta_time: f32,
        event_receiver: SharedMutex<dyn MidiEventReceiver>,
    ) -> Result<(), Error> {
        let mut receiver = event_receiver.lock().unwrap();
        match self.state {
            PlaybackState::Playing(mut t) => {
                let data = self
                    .data
                    .as_ref()
                    .ok_or("Cannot get midi data for playback")?;
                t = self.time + delta_time;
                let playback_time_ticks: u32 = (t * self.tps) as u32;
                let playback_midi_ticks: u32 = self.midi_ticks;
                let delta_ticks = playback_time_ticks - playback_midi_ticks;
                let mut last_event_ticks: u32 = playback_midi_ticks;
                let mut current_ticks: u32 = 0;

                for track in data.tracks.iter().skip(1) {
                    for event in track.iter() {
                        current_ticks += event.delta;
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
            _ => Ok(()),
        }
    }

    fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm;
        self.tps = Self::calculate_tps(self.bpm, self.ppq);
    }

    fn reset(&mut self) {
        self.data = None;
        self.time = 0.0;
        self.midi_ticks = 0;
        self.state = PlaybackState::None;
    }

    fn load(&mut self, data: Smf<'_>) {
        self.data = Some(OwnedSmf::try_from(&data).unwrap());
        let ppq = match data.header.timing {
            Timing::Metrical(v) => v.into(),
            Timing::Timecode(_, _) => 192,
        };
        self.ppq = ppq as u32;
        self.bpm = 120.0;
        self.tps = Self::calculate_tps(self.bpm, self.ppq);
        self.midi_ticks = 0;
        self.time = 0.0;
        self.state = PlaybackState::Stoped;
    }

    fn play(&mut self) {
        self.state = PlaybackState::Playing(self.time);
    }

    fn get_state(&self) -> PlaybackState {
        self.state
    }
}
