use crate::error::Error;

pub struct AudioBufferMono {
    samples: Vec<f32>,
    extrema: (f32, f32),
}

pub struct AudioBuffer {
    channels: u8,
    buffers: Vec<AudioBufferMono>,
}

pub struct AudioBufferBuilder {
    channels: Option<u8>,
    samples: Option<usize>,
}

impl AudioBufferBuilder {
    pub fn new() -> Self {
        Self {
            channels: None,
            samples: None,
        }
    }

    pub fn set_channels(&mut self, n: u8) -> &mut Self {
        self.channels = Some(n);
        self
    }

    pub fn set_samples(&mut self, n: usize) -> &mut Self {
        self.samples = Some(n);
        self
    }

    pub fn build(self) -> Result<AudioBuffer, Error> {
        let channels = self.channels.ok_or(Error::Specify("channels count"))?;
        let samples = self.samples.ok_or(Error::Specify("samples count"))?;
        Ok(AudioBuffer {
            channels,
            buffers: std::iter::repeat_with(|| AudioBufferMono::new(samples))
                .take(channels.into())
                .collect::<Vec<_>>(),
        })
    }
}

impl AudioBufferMono {
    pub fn new(s: usize) -> Self {
        Self {
            samples: vec![0.0; s],
            extrema: (0.0, 0.0),
        }
    }
}

impl Default for AudioBufferBuilder {
    fn default() -> Self {
        Self::new()
    }
}
