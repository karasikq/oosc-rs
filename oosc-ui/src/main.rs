pub mod app;

use anyhow::Error;
use cpal::traits::StreamTrait;
use midly::Smf;
use oosc_core::midi::playback::{PlaybackControl, SmfPlayback};

use self::app::application::Application;

fn main() -> Result<(), Error> {
    let mut app = Application::new()?;

    {
        let smf = Smf::parse(include_bytes!(
            "../test-resources/midi/Beethoven-Moonlight-Sonata.mid"
        ))
        .unwrap();
        let mut midi_playback = SmfPlayback::new(smf)?;
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
