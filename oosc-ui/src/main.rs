pub mod app;
pub mod ui;

use self::app::application::Application;
use anyhow::Error;
use cpal::traits::StreamTrait;
use midly::Smf;
use oosc_core::core::oscillator::WavetableOscillator;

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
        let syn = app.ctx.synthesizer.lock().unwrap();
        let mut oscs = syn.get_oscillators::<WavetableOscillator>();
        let osc = oscs.next().unwrap();
        let mut osc = osc.write().unwrap();
        let osc = osc
            .as_any_mut()
            .downcast_mut::<WavetableOscillator>()
            .unwrap();
        osc.wavetable()
            .write()
            .unwrap()
            .load_from(
                "./oosc-ui/test-wavetables/2/SX 09.wav",
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
                "./oosc-ui/test-wavetables/2/OB2v 01.wav",
                2048,
            )
            .unwrap();
    }

    let stream = app.detach_stream()?;
    stream.play().unwrap();
    app.run()?;
    Ok(())
}
