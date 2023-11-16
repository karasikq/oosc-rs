use crate::error::Error;

pub struct SampleBufferMono {
    samples: Vec<f32>,
    extrema: (f32, f32),
}

pub struct SampleBuffer {
    channels: u8,
    buffers: Vec<SampleBufferMono>,
    samples_count: usize,
}

pub struct SampleBufferBuilder {
    channels: Option<u8>,
    samples: Option<usize>,
}

impl SampleBufferBuilder {
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

    pub fn from_array(self, a: &[f32]) -> SampleBuffer {
        SampleBuffer {
            channels: 1,
            buffers: vec![SampleBufferMono::from_array(a)],
            samples_count: a.len(),
        }
    }

    pub fn from_vec(self, a: Vec<f32>) -> SampleBuffer {
        SampleBuffer {
            channels: 1,
            samples_count: a.len(),
            buffers: vec![SampleBufferMono::from_vec(a)],
        }
    }

    pub fn build(self) -> Result<SampleBuffer, Error> {
        let channels = self.channels.ok_or(Error::Specify("channels count"))?;
        let samples = self.samples.ok_or(Error::Specify("samples count"))?;
        Ok(SampleBuffer {
            channels,
            buffers: std::iter::repeat_with(|| SampleBufferMono::new(samples))
                .take(channels.into())
                .collect::<Vec<_>>(),
            samples_count: samples,
        })
    }
}

impl SampleBufferMono {
    pub fn new(s: usize) -> Self {
        Self {
            samples: vec![0.0; s],
            extrema: (0.0, 0.0),
        }
    }

    pub fn at(&self, index: usize) -> Result<f32, Error> {
        let sample = self
            .samples
            .get(index)
            .ok_or(Error::from("Index of sample out of buffer"))?;
        Ok(*sample)
    }

    pub fn from_array(a: &[f32]) -> Self {
        let samples = a.to_vec();
        Self {
            samples,
            extrema: Self::extremas(a),
        }
    }

    pub fn from_vec(a: Vec<f32>) -> Self {
        let samples = a;
        Self {
            extrema: Self::extremas(&samples),
            samples,
        }
    }

    fn extremas(a: &[f32]) -> (f32, f32) {
        let (mut min, mut max) = (f32::MAX, f32::MIN);
        for v in a {
            if *v > max {
                max = *v;
            }
            if *v < min {
                min = *v;
            }
        }
        (min, max)
    }
}

impl SampleBuffer {
    pub fn len(&self) -> usize {
        self.samples_count
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn at(&self, index: usize, channel: u8) -> Result<f32, Error> {
        let sample = self
            .buffers
            .get(channel as usize)
            .ok_or(Error::from("Channel index out of buffer size"))?
            .at(index)?;
        Ok(sample)
    }
}

impl Default for SampleBufferBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::sample_buffer::SampleBufferMono;

    #[test]
    fn test_mono_buffer() {
        let buffer = SampleBufferMono::from_array(&[0.1, 0.2, 0.95, -0.93, -0.934]);
        assert_eq!(buffer.extrema, (-0.934, 0.95));
    }

    #[test]
    fn test_buffer_at() {
        let buffer = SampleBufferMono::from_array(&[0.1, 0.2, 0.95, -0.93, -0.934]);
        if let Ok(v) = buffer.at(1) {
            assert_eq!(v, 0.2)
        };
        if buffer.at(100).is_ok() {
            panic!("Index should be out of range")
        };
    }
}
