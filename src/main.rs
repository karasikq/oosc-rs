pub mod app;
pub mod core;
pub mod effects;
pub mod error;
pub mod utils;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use self::error::Error;

use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::core::synthesizer::SynthesizerBuilder;

use self::{
    core::{
        note::Note,
        oscillator::{OscillatorBuilder, WavetableOscillator},
        synthesizer::Synthesizer,
        waveshape::WaveShape,
        wavetable::WaveTableBuilder,
    },
    utils::{
        adsr_envelope::{ADSREnvelopeBuilder, State},
        sample_buffer::SampleBufferBuilder,
    },
};

// need to load sample rate from device info
static BUFFER_SIZE: usize = 512;
static SAMPLE_RATE: u32 = 48000;

fn main() -> Result<(), Error> {
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

    let device = host
        .default_output_device()
        .expect("failed to find input device");

    let config = cpal::StreamConfig {
        channels: 2,
        sample_rate: cpal::SampleRate(SAMPLE_RATE),
        buffer_size: cpal::BufferSize::Fixed(512),
    };

    let syn = create_syn();
    let syn2 = syn.clone();
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let time = Arc::new(Mutex::new(0.0));
    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut s = syn.lock().unwrap();
                let buf = s.output().unwrap();
                let mut b = buf.iter(0).unwrap();
                for frame in data.chunks_exact_mut(2) {
                    let s = b.next().unwrap();
                    for f in frame.iter_mut() {
                        *f = s;
                    }
                    let mut t = time.lock().unwrap();
                    *t += 1.0 / SAMPLE_RATE as f32;
                }

                println!("{}", time.lock().unwrap());
            },
            err_fn,
            None,
        )
        .unwrap();
    stream.play().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1000));
    {
        let mut s = syn2.lock().unwrap();
        play_note(&mut s, 60);
    }
    std::thread::sleep(std::time::Duration::from_millis(500));
    {
        let mut s = syn2.lock().unwrap();
        play_note(&mut s, 63);
    }
    std::thread::sleep(std::time::Duration::from_millis(200));
    {
        let mut s = syn2.lock().unwrap();
        play_note(&mut s, 64);
    }
    std::thread::sleep(std::time::Duration::from_millis(5000));
    test_render();
    Ok(())
}

fn test_render() {
    let synthesizer = create_syn();
    let mut syn = synthesizer.lock().unwrap();
    play_note(&mut syn, 60);

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let now = Instant::now();
    let mut writer = hound::WavWriter::create("output.wav", spec).unwrap();
    let chunks = SAMPLE_RATE as usize * 13 / BUFFER_SIZE;
    for _ in 0..chunks {
        let s = syn.output().unwrap();
        for i in 0..BUFFER_SIZE {
            let _ = writer.write_sample((s.at(0, i).unwrap() * i16::MAX as f32) as i16);
        }
    }
    let _ = writer.flush();
    let _ = writer.finalize();
    println!("{}", now.elapsed().as_millis());
}

fn create_syn() -> Arc<Mutex<Synthesizer>> {
    let osc1 = create_osc(WaveShape::Sin);
    Arc::new(Mutex::new(
        SynthesizerBuilder::new()
            .set_buffer(BUFFER_SIZE)
            .unwrap()
            .add_osc(Box::new(osc1))
            .set_sample_rate(SAMPLE_RATE)
            .build()
            .unwrap(),
    ))
}

fn create_osc(shape: WaveShape) -> WavetableOscillator {
    let adsr = ADSREnvelopeBuilder::new()
        .set_attack(1., 1.)
        .unwrap()
        .set_decay(0.5, 0.9)
        .unwrap()
        .set_release(3.)
        .unwrap()
        .build()
        .unwrap();
    let buffer = SampleBufferBuilder::new()
        .set_channels(2)
        .set_samples(BUFFER_SIZE)
        .build()
        .unwrap();
    let table = WaveTableBuilder::new()
        .from_shape(shape, BUFFER_SIZE)
        .set_interpolation(utils::interpolation::InterpolateMethod::Linear)
        .build()
        .unwrap();
    OscillatorBuilder::new()
        .set_buffer(buffer)
        .set_envelope(adsr)
        .set_wavetable(table)
        .build()
        .unwrap()
}

fn play_note(syn: &mut Synthesizer, note: i32) -> &mut Synthesizer {
    let mut note = Note::new(note, 127);
    note.hold_on = State::None;
    syn.note_on(note).unwrap();
    syn
}
