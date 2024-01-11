use std::{
    fs::File,
    io::BufWriter,
    sync::{Arc, Mutex},
};

use hound::{WavSpec, WavWriter};

use crate::{
    error::Error,
    utils::{make_shared, sample_buffer::BufferSettings, Shared},
};

use super::StreamCallback;

#[derive(Clone, Copy)]
pub enum RenderState {
    None,
    Rendering,
}

pub trait StreamRenderer: Sync + Send {
    fn start(&mut self) -> Result<(), Error>;
    fn stop(&mut self) -> Result<(), Error>;
    fn record(&mut self, samples: &[f32], sample_rate: f32) -> Result<(), Error>;
    fn reset(&mut self);
    fn get_state(&self) -> RenderState;
    fn time(&self) -> f32;
}

type Writer = WavWriter<BufWriter<File>>;

pub struct StreamWavRenderer {
    writer: Shared<Option<Writer>>,
    spec: WavSpec,
    state: RenderState,
    time: f32,
}

impl StreamWavRenderer {
    pub fn new(spec: WavSpec) -> Self {
        Self {
            writer: make_shared(None),
            spec,
            state: RenderState::None,
            time: 0.0,
        }
    }

    pub fn to_file<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<&mut Self, Error> {
        self.writer = make_shared(Some(
            WavWriter::create(path, self.spec).map_err(|e| e.to_string())?,
        ));
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
    fn start(&mut self) -> Result<(), Error> {
        let writer_guard = self.writer.read().unwrap();
        if writer_guard.is_none() {
            return Err(Error::Generic("Cannot get any writer".to_string()))?;
        }
        self.state = RenderState::Rendering;
        self.time = 0.0;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), Error> {
        self.state = RenderState::None;
        let writer = self
            .writer
            .write()
            .unwrap()
            .take()
            .ok_or("Cannot get wav writer")?;
        writer.finalize().map_err(|e| e.to_string())?;
        self.reset();
        Ok(())
    }

    fn record(&mut self, samples: &[f32], sample_rate: f32) -> Result<(), Error> {
        let mut writer_guard = self.writer.write().unwrap();
        let writer = writer_guard.as_mut().ok_or("Cannot get wav writer")?;
        for s in samples {
            writer.write_sample(*s).map_err(|e| e.to_string())?;
        }
        self.time += samples.len() as f32 / sample_rate;
        Ok(())
    }

    fn reset(&mut self) {
        *self.writer.write().unwrap() = None;
        self.state = RenderState::None;
    }

    fn get_state(&self) -> RenderState {
        self.state
    }

    fn time(&self) -> f32 {
        self.time
    }
}

pub struct RenderStreamCallback(pub Arc<Mutex<dyn StreamRenderer>>);

impl StreamCallback for RenderStreamCallback {
    fn process_stream(
        &mut self,
        data: &mut [f32],
        _time: f32,
        sample_rate: f32,
    ) -> std::result::Result<(), Error> {
        let mut renderer = self.0.lock().unwrap();
        if let RenderState::None = renderer.get_state() {
            return Ok(());
        }
        renderer.record(data, sample_rate)?;
        Ok(())
    }
}
