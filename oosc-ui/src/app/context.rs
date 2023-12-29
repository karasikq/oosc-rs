use anyhow::Result;
use crossterm::{
    execute,
    terminal::{self, enable_raw_mode, EnterAlternateScreen},
};
use std::{
    io::Stdout,
    sync::{Arc, Mutex, RwLock},
};

use cpal::{
    traits::{DeviceTrait, HostTrait},
    Device, Host, StreamConfig,
};

use oosc_core::{
    callbacks::{
        stream_callback::{MidiStreamCallback, SynthesizerStreamCallback},
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
    midi::playback::{MidiPlayback, SmfPlayback},
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
    pub synthesizer_callback: SharedMutex<SynthesizerStreamCallback>,
    pub midi_callback: SharedMutex<MidiStreamCallback>,
}

impl CallbacksData {
    pub fn get_callbacks(&self) -> Vec<Arc<Mutex<dyn StreamCallback>>> {
        vec![
            self.synthesizer_callback.clone(),
            self.midi_callback.clone(),
        ]
    }
}

pub struct Context {
    pub synthesizer: SharedMutex<Synthesizer>,
    pub callbacks: CallbacksData,
    pub midi_control: SharedMutex<dyn MidiPlayback>,
    pub terminal: AppTerminal,
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
                .add_effect(delay)
                .add_effect(compressor)
                .set_sample_rate(config.sample_rate)
                .build()?,
        ));

        let synthesizer_cloned = synthesizer.clone();
        let synthesizer_callback = make_shared_mutex(SynthesizerStreamCallback(synthesizer_cloned));

        let synthesizer_cloned = synthesizer.clone();
        let midi_control = Arc::new(Mutex::new(SmfPlayback::default()));
        let midi_control_cloned = midi_control.clone();
        let midi_callback = Arc::new(Mutex::new(MidiStreamCallback(
            midi_control_cloned,
            synthesizer_cloned,
        )));

        let callbacks = CallbacksData {
            synthesizer_callback,
            midi_callback,
        };
        let terminal = build_terminal()?;
        setup_panic_hook();

        Ok(Self {
            synthesizer,
            callbacks,
            midi_control,
            terminal,
        })
    }

    pub fn get_default_device(config: &Config) -> Result<(Host, Device, StreamConfig), Error> {
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

        let device = host
            .default_output_device()
            .ok_or("Cannot get default output device")?;
        println!("{}", device.name().unwrap());

        let config = cpal::StreamConfig {
            channels: config.channels as u16,
            sample_rate: cpal::SampleRate(config.sample_rate),
            buffer_size: cpal::BufferSize::Fixed(config.buffer_size as u32),
        };
        println!("{:?}", config);

        Ok((host, device, config))
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
}

fn build_terminal() -> Result<AppTerminal> {
    let mut stdout = std::io::stdout();
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
