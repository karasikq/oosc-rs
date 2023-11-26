use std::sync::{Arc, Mutex};

use cpal::{traits::{HostTrait, DeviceTrait}, Device, StreamConfig, Host};

use crate::{
    core::{
        oscillator::{OscillatorBuilder, WavetableOscillator},
        synthesizer::{Synthesizer, SynthesizerBuilder},
        waveshape::WaveShape,
        wavetable::WaveTableBuilder,
    },
    error::Error,
    utils::{
        adsr_envelope::ADSREnvelope, interpolation::InterpolateMethod,
        sample_buffer::SampleBufferBuilder,
    },
};

use super::config::Config;

pub struct Context {
    pub synthesizer: Arc<Mutex<Synthesizer>>,
    /* pub device: Device,
    pub stream_config: StreamConfig, */
}

impl Context {
    pub fn build_default(config: &Config) -> Result<Self, Error> {
        let osc1 = Self::build_osc(config)?;
        let osc2 = Self::build_osc(config)?;
        let synthesizer = Arc::new(Mutex::new(
            SynthesizerBuilder::new()
                .set_buffer(config.buffer_size)?
                .add_osc(osc1)
                .add_osc(osc2)
                .set_sample_rate(config.sample_rate)
                .build()?,
        ));
        // let (device, stream_config) = Self::get_default_device(config)?;
        Ok(Self {
            synthesizer,
            /* device,
            stream_config, */
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
            buffer_size: cpal::BufferSize::Fixed(config.buffer_size as u32 - 42),
        };
        println!("{:?}", config);

        Ok((host, device, config))
    }

    fn build_osc(config: &Config) -> Result<Box<WavetableOscillator>, Error> {
        let adsr = ADSREnvelope::default();
        let buffer = SampleBufferBuilder::new()
            .set_channels(2)
            .set_samples(config.buffer_size)
            .build()?;
        let table = WaveTableBuilder::new()
            .from_shape(WaveShape::Sin, config.buffer_size)
            .set_interpolation(InterpolateMethod::Linear)
            .build()?;
        Ok(Box::new(
            OscillatorBuilder::new()
                .set_buffer(buffer)
                .set_envelope(adsr)
                .set_wavetable(table)
                .build()?,
        ))
    }
}
