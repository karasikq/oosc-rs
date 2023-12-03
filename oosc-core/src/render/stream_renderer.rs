use std::{fs::File, io::BufWriter};

use hound::{WavSpec, WavWriter};

use crate::error::Error;

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
