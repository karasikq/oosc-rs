pub mod app;
pub mod ui;

use self::app::application::Application;
use anyhow::Error;
use cpal::traits::StreamTrait;
use midly::Smf;

fn main() -> Result<(), Error> {
    let mut app = Application::new()?;
    {
        let mut midi_control = app.ctx.midi_control.lock().unwrap();
        let smf = Smf::parse(include_bytes!(
            "../test-resources/midi/Gravity Falls - Made Me Realize.mid"
        ))
        .unwrap();
        midi_control.load(smf);
        // midi_control.set_bpm(69.0);
        midi_control.play();
    }

    let stream = app.detach_stream()?;
    stream.play().unwrap();
    app.run()?;
    Ok(())
}
