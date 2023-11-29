use cpal::{traits::{DeviceTrait, StreamTrait}, Stream};

use crate::app::context;

use self::{
    app::application::{Application},
    error::Error,
    midi::{mediator::MidiSynthesizerMediator, playback::Playback},
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

    // MIDI file
    let smf = Smf::parse(include_bytes!(
        "../resources/midi/Beethoven-Moonlight-Sonata.mid"
    ))
    .unwrap();
    let mut midi_playback = Playback::new(smf);
    midi_playback.set_bpm(69.0);

    let stream = app.detach_stream()?;
    stream.play().unwrap();
    app.run()?;
    // std::thread::sleep(std::time::Duration::from_millis(60000));
    Ok(())
}

