use std::{thread, time::Duration};

use cpal::traits::{DeviceTrait, StreamTrait};

use crate::app::context;

use self::{
    app::application::Application,
    core::{note::Note, synthesizer::Synthesizer},
    error::Error,
};
use midly::{Smf, Timing};

pub mod app;
pub mod core;
pub mod effects;
pub mod error;
pub mod utils;

fn main() -> Result<(), Error> {
    let mut app = Application::new()?;
    let (_, device, config) = context::Context::get_default_device(&app.config)?;
    let err_fn = |err| println!("an error occurred on stream: {}", err);
    let syn = app.ctx.synthesizer.clone();

    // MIDI file
    let smf = Smf::parse(include_bytes!("../test.mid")).unwrap();
    let mut midi_total_ticks: u32 = 0;

    let mut total_playback_seconds = 0.;
    let delta = app.config.buffer_size as f32 / app.config.sample_rate as f32;
    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                {
                    let mut s = syn.lock().unwrap();
                    let buf = s.output();
                    if buf.is_err() {
                        println!("{}", buf.err().unwrap().to_string());
                        return;
                    }
                    let buf = buf.unwrap();
                    let mut b = buf.iter(0).unwrap();
                    for frame in data.chunks_exact_mut(2) {
                        let s = b.next().unwrap();
                        for f in frame.iter_mut() {
                            *f = s;
                        }
                    }
                }

                total_playback_seconds += delta;

                {
                    let mut s = syn.lock().unwrap();
                    check_midi(&smf, &mut midi_total_ticks, total_playback_seconds, &mut s)
                        .unwrap();
                }
            },
            err_fn,
            None,
        )
        .map_err(|e| e.to_string())?;
    stream.play().map_err(|e| e.to_string())?;
    app.run()?;
    // thread::sleep(Duration::from_millis(60000));
    Ok(())
}

fn check_midi(
    data: &Smf,
    midi_time: &mut u32,
    time: f32,
    syn: &mut Synthesizer,
) -> Result<(), Error> {
    let ppq = match data.header.timing {
        Timing::Metrical(v) => v.into(),
        Timing::Timecode(_, _) => 192,
    };
    let tps = (120.0 * ppq as f32) / 60.;

    let playback_time_ticks: u32 = (time * tps) as u32;
    let playback_midi_ticks: u32 = *midi_time;
    let delta_ticks = playback_time_ticks - playback_midi_ticks;
    let mut last_event_ticks: u32 = playback_midi_ticks;
    let mut current_ticks: u32 = 0;

    for track in data.tracks.iter().skip(1) {
        for event in track.iter() {
            current_ticks += event.delta.as_int();
            if current_ticks > playback_midi_ticks + delta_ticks {
                *midi_time = match last_event_ticks {
                    0 => 1,
                    v => v,
                };
                return Ok(());
            }
            if current_ticks > playback_midi_ticks
                || (current_ticks == 0 && playback_midi_ticks == 0)
            {
                log(format!(
                    "MidiTick {}. Ticks {}. Kind {:?}\n",
                    *midi_time, current_ticks, event.kind
                ));
                match event.kind {
                    midly::TrackEventKind::Midi { channel, message } => match message {
                        midly::MidiMessage::NoteOn { key, vel } => {
                            let key = key.as_int();
                            let vel = vel.as_int();
                            syn.note_on(Note::new(key.into(), vel.into()))?;
                        }
                        midly::MidiMessage::NoteOff { key, .. } => {
                            let key = key.as_int();
                            let action = syn.note_off(key.into());
                            match action {
                                Ok(_) => (),
                                Err(e) => println!("{}", e.to_string()),
                            }
                        }
                        _ => (),
                    },
                    midly::TrackEventKind::SysEx(_) => (),
                    midly::TrackEventKind::Escape(_) => (),
                    midly::TrackEventKind::Meta(_) => (),
                };
                last_event_ticks = current_ticks;
            }
        }
    }

    *midi_time = match last_event_ticks {
        0 => 1,
        v => v,
    };
    Ok(())
}

fn log(str: String) {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open("log.txt")
        .unwrap();
    std::io::Write::write(&mut file, str.as_bytes()).unwrap();
}
