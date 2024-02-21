use anyhow::Result;
use crossterm::{
    event::{KeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
    terminal::{self, enable_raw_mode, EnterAlternateScreen},
};
use midir::{Ignore, MidiInput, MidiInputConnection};
use midly::live::LiveEvent;
use std::{
    io::Stdout,
    sync::{Arc, Mutex, RwLock},
};

use cpal::{
    traits::{DeviceTrait, HostTrait},
    Device, Host, SupportedStreamConfig,
};

use oosc_core::{
    callbacks::{
        stream_callback::{MidiStreamCallback, SynthesizerStreamCallback},
        stream_renderer::{RenderStreamCallback, StreamWavRenderer},
        StreamCallback,
    },
    core::{
        oscillator::{OscillatorBuilder, WavetableOscillator},
        synthesizer::{Synthesizer, SynthesizerBuilder},
        waveshape::WaveShape,
        wavetable::WaveTableBuilder,
    },
    effects::{amplifier::Amplifier, chorus::Chorus, compressor::Compressor, delay::Delay},
    error::Error,
    midi::{
        mediator::{MidiEventReceiver, MidiSynthesizerMediator},
        playback::{MidiPlayback, SmfPlayback},
        smf_extensions::OwnedTrackEvent,
    },
    utils::{
        adsr_envelope::ADSREnvelope,
        interpolation::InterpolateMethod,
        make_shared, make_shared_mutex,
        sample_buffer::{BufferSettings, SampleBufferBuilder},
        Shared, SharedMutex,
    },
};
use ratatui::{prelude::CrosstermBackend, Terminal};

use super::config::Config;

type AppTerminal = Shared<Terminal<CrosstermBackend<Stdout>>>;

pub struct CallbacksData {
    pub output: SharedMutex<SynthesizerStreamCallback>,
    pub smf: SharedMutex<MidiStreamCallback>,
    pub render: SharedMutex<RenderStreamCallback>,
}

impl CallbacksData {
    pub fn get_callbacks(&self) -> Vec<Arc<Mutex<dyn StreamCallback>>> {
        vec![self.output.clone(), self.smf.clone(), self.render.clone()]
    }
}

pub struct MidiContext {
    pub input: Shared<MidiInputConnection<()>>,
}

pub struct Context {
    pub synthesizer: SharedMutex<Synthesizer>,
    pub callbacks: CallbacksData,
    pub midi_control: SharedMutex<dyn MidiPlayback>,
    pub render_control: SharedMutex<StreamWavRenderer>,
    pub terminal: AppTerminal,
    pub midi: Option<MidiContext>,
}

impl Context {
    pub fn build_default(config: &Config) -> Result<Self> {
        let settings = BufferSettings {
            samples: config.buffer_size,
            channels: config.channels as usize,
            sample_rate: config.sample_rate as f32,
        };
        let osc1 = Self::build_osc(config, WaveShape::Sin)?;
        let osc2 = Self::build_osc(config, WaveShape::Triangle)?;
        let chorus = make_shared(Chorus::default(&settings));
        let compressor = make_shared(Compressor::default(&settings));
        let delay = make_shared(Delay::default(&settings));
        let amplifier = make_shared(Amplifier::default());
        let synthesizer = Arc::new(Mutex::new(
            SynthesizerBuilder::new()
                .set_buffer(config.buffer_size)?
                .add_osc(osc1)
                .add_osc(osc2)
                .add_effect(amplifier)
                .add_effect(chorus)
                // .add_effect(delay)
                // .add_effect(compressor)
                .set_sample_rate(config.sample_rate)
                .build()?,
        ));

        let synthesizer_callback =
            make_shared_mutex(SynthesizerStreamCallback(synthesizer.clone()));

        let midi_mediator = make_shared_mutex(MidiSynthesizerMediator::new(synthesizer.clone()));
        let midi_control = make_shared_mutex(SmfPlayback::default());
        let midi_control_cloned = midi_control.clone();
        let midi_callback = make_shared_mutex(MidiStreamCallback(
            midi_control_cloned,
            midi_mediator.clone(),
        ));

        let render_control = make_shared_mutex(StreamWavRenderer::from(&settings));
        let render_callback = make_shared_mutex(RenderStreamCallback(render_control.clone()));

        let callbacks = CallbacksData {
            output: synthesizer_callback,
            smf: midi_callback,
            render: render_callback,
        };

        let midi_input = Self::build_midi_input(midi_mediator);
        let midi = match midi_input.is_ok() {
            true => {
                let input = make_shared(midi_input.unwrap());
                Some(MidiContext { input })
            }
            false => None,
        };

        let terminal = build_terminal()?;
        setup_panic_hook();

        Ok(Self {
            synthesizer,
            callbacks,
            midi_control,
            render_control,
            terminal,
            midi,
        })
    }

    pub fn get_default_device() -> Result<(Host, Device, SupportedStreamConfig), Error> {
        #[cfg(any(
            not(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd"
            )),
            not(feature = "jack")
        ))]
        let host = cpal::default_host();
        println!("{}", host.id().name());

        for dev in host.output_devices().unwrap() {
            println!("{}", dev.name().unwrap());

            let default_config = dev
                .default_output_config()
                .expect("Cannot get default output device");
            println!("{:?}", default_config);
        }
        let device = host
            .default_output_device()
            .ok_or("Cannot get default output device")?;
        println!("{}", device.name().unwrap());

        let default_config = device
            .default_output_config()
            .expect("Cannot get default output device");
        println!("{:?}", default_config);

        Ok((host, device, default_config))
    }

    fn build_osc(
        config: &Config,
        shape: WaveShape,
    ) -> Result<Arc<RwLock<WavetableOscillator>>, Error> {
        let adsr = ADSREnvelope::default();
        let buffer = SampleBufferBuilder::new()
            .set_channels(2)
            .set_samples(config.buffer_size)
            .build()?;
        let table = WaveTableBuilder::new()
            .from_shape(shape, config.buffer_size)
            .set_interpolation(InterpolateMethod::Linear)
            .build()?;
        Ok(make_shared(
            OscillatorBuilder::new()
                .set_buffer(buffer)
                .set_envelope(adsr)
                .set_wavetable(table)
                .build()?,
        ))
    }

    fn build_midi_input(
        mediator: SharedMutex<dyn MidiEventReceiver>,
    ) -> Result<MidiInputConnection<()>, anyhow::Error> {
        let mut midi_in = MidiInput::new("oosc")?;
        midi_in.ignore(Ignore::None);
        let binding = midi_in.ports();
        let in_port = anyhow::Context::context(binding.get(0), "Cannot get MIDI-IN port")?;
        let mediator = mediator.clone();
        let connection = midi_in
            .connect(
                in_port,
                "oosc-input",
                move |_stamp, message, _| {
                    let event =
                        OwnedTrackEvent::try_from(&LiveEvent::parse(message).unwrap()).unwrap();
                    let mut receiver = mediator.lock().unwrap();
                    receiver.receive_event(&event).unwrap();
                },
                (),
            )
            .map_err(|e| {
                anyhow::anyhow!(format!(
                    "Cannot connect to MIDI-IN port. Reason: {}",
                    e.to_string()
                ))
            })?;
        Ok(connection)
    }
}

fn build_terminal() -> Result<AppTerminal> {
    let mut stdout = std::io::stdout();
    execute!(
        stdout,
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
    )?;
    anyhow::Context::context(enable_raw_mode(), "Failed to enable raw mode")?;
    anyhow::Context::context(
        execute!(stdout, EnterAlternateScreen),
        "Unable to enter alternate screen",
    )?;
    Ok(make_shared(anyhow::Context::context(
        Terminal::new(CrosstermBackend::new(stdout)),
        "Creating terminal failed",
    )?))
}

pub fn restore_terminal() -> Result<(), anyhow::Error> {
    execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic| {
        let restore = restore_terminal();
        if restore.is_err() {
            println!(
                "Error until restore terminal on panic: {}",
                restore.err().unwrap()
            );
        }
        original_hook(panic);
    }));
}
