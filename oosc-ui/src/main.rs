pub mod app;
pub mod ui;

use anyhow::Error;
use cpal::traits::StreamTrait;
use midly::Smf;
use oosc_core::core::{oscillator::WavetableOscillator, parametrs::Parametr};

use self::app::application::Application;

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

    {
        let mut syn = app.ctx.synthesizer.lock().unwrap();
        let mut oscs = syn.get_oscillators::<WavetableOscillator>();
        let osc1 = oscs.next().unwrap();
        let mut osc1 = osc1.write().unwrap();
        let osc1 = osc1
            .as_any_mut()
            .downcast_mut::<WavetableOscillator>()
            .unwrap();
        osc1.wavetable()
            .write()
            .unwrap()
            .load_from(
                "./oosc-ui/test-resources/wavetables/2457-Veridian's Tables/McFly.wav",
                2048,
            )
            .unwrap();

        let osc1 = oscs.next().unwrap();
        let mut osc1 = osc1.write().unwrap();
        let osc1 = osc1
            .as_any_mut()
            .downcast_mut::<WavetableOscillator>()
            .unwrap();
        osc1.wavetable()
            .write()
            .unwrap()
            .load_from(
                "./oosc-ui/test-resources/wavetables/patchmaker/BFC12.wav",
                2048,
            )
            .unwrap();
    }

    let stream = app.detach_stream()?;
    stream.play().unwrap();
    app.run()?;
    // std::thread::sleep(std::time::Duration::from_millis(60000));
    Ok(())
}
