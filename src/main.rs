use cpal::traits::{DeviceTrait, StreamTrait};

use crate::app::context;

use self::{
    app::application::Application,
    core::note::Note,
    error::Error,
    midi::playback::Playback,
};
use midly::Smf;

pub mod app;
pub mod core;
pub mod effects;
pub mod error;
pub mod midi;
pub mod utils;

fn main() -> Result<(), Error> {
    let mut app = Application::new()?;
    let (_, device, config) = context::Context::get_default_device(&app.config)?;
    let err_fn = |err| println!("an error occurred on stream: {}", err);
    let syn = app.ctx.synthesizer.clone();

    // MIDI file
    let smf = Smf::parse(include_bytes!("../resources/midi/Beethoven-Moonlight-Sonata.mid")).unwrap();
    let mut midi_playback = Playback::new(smf);
    midi_playback.set_bpm(69.0);

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
                    midi_playback
                        .play(total_playback_seconds, |event| -> Result<(), Error> {
                            match event.kind {
                                midly::TrackEventKind::Midi { message, .. } => match message {
                                    midly::MidiMessage::NoteOn { key, vel } => {
                                        let key = key.as_int();
                                        let vel = vel.as_int();
                                        s.note_on(Note::new(key.into(), vel.into()))?;
                                    }
                                    midly::MidiMessage::NoteOff { key, .. } => {
                                        let key = key.as_int();
                                        let action = s.note_off(key.into());
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
                            Ok(())
                        })
                        .unwrap();
                }
            },
            err_fn,
            None,
        )
        .map_err(|e| e.to_string())?;
    stream.play().map_err(|e| e.to_string())?;
    app.run()?;
    // std::thread::sleep(std::time::Duration::from_millis(60000));
    Ok(())
}
