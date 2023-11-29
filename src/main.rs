use cpal::{
    traits::{DeviceTrait, StreamTrait},
    Stream,
};

use crate::app::context;

use self::{
    app::application::Application,
    error::Error,
    midi::{mediator::MidiSynthesizerMediator, playback::{SmfPlayback, PlaybackControl}},
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
    {
        let mut midi_playback = SmfPlayback::new(smf);
        midi_playback.set_bpm(69.0);
        let midi_control = app.ctx.midi_control.clone();
        let mut midi_control = midi_control.lock().unwrap();
        *midi_control = Some(Box::new(midi_playback));
    }

    let stream = app.detach_stream()?;
    stream.play().unwrap();
    app.run()?;
    // std::thread::sleep(std::time::Duration::from_millis(60000));
    Ok(())
}
