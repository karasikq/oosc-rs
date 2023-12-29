use std::{
    fs::File,
    io::BufWriter,
    sync::{Arc, Mutex},
};

use hound::{WavSpec, WavWriter};

use crate::{error::Error, utils::sample_buffer::BufferSettings};

use super::StreamCallback;

#[derive(Clone, Copy)]
pub enum RenderState {
    None,
    Rendering,
}

pub trait StreamRenderer: Sync + Send {
    fn start(&mut self);
    fn stop(&mut self);
    fn record(&mut self, samples: &[f32]) -> Result<(), Error>;
    fn reset(&mut self);
    fn get_state(&self) -> RenderState;
}

type Writer = WavWriter<BufWriter<File>>;

pub struct StreamWavRenderer {
    writer: Option<Writer>,
    spec: WavSpec,
    state: RenderState,
}

impl StreamWavRenderer {
    pub fn new(spec: WavSpec) -> Self {
        Self {
            writer: None,
            spec,
            state: RenderState::None,
        }
    }

    pub fn to_file<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<&mut Self, Error> {
        self.writer = Some(WavWriter::create(path, self.spec).map_err(|e| e.to_string())?);
        Ok(self)
    }
}

impl<'a, T> From<T> for StreamWavRenderer
where
    T: Into<&'a BufferSettings>,
{
    fn from(value: T) -> Self {
        let settings = value.into();
        Self::new(WavSpec {
            channels: settings.channels as u16,
            sample_rate: settings.sample_rate as u32,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        })
    }
}

impl StreamRenderer for StreamWavRenderer {
    fn start(&mut self) {
        self.state = RenderState::Rendering;
    }

    fn stop(&mut self) {
        self.state = RenderState::None;
    }

    fn record(&mut self, samples: &[f32]) -> Result<(), Error> {
        let writer = self.writer.as_mut().ok_or("Cannot get wav writer")?;
        for s in samples {
            writer.write_sample(*s).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.writer = None;
        self.state = RenderState::None;
    }

    fn get_state(&self) -> RenderState {
        self.state
    }
}

pub struct RenderStreamCallback(pub Arc<Mutex<dyn StreamRenderer>>);

impl StreamCallback for RenderStreamCallback {
    fn process_stream(&mut self, data: &mut [f32], _time: f32) -> std::result::Result<(), Error> {
        let mut renderer = self.0.lock().unwrap();
        if let RenderState::None = renderer.get_state() {
            return Ok(());
        }
        renderer.record(data)?;
        Ok(())
    }
}
