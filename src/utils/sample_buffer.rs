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

    pub fn from_array(a: &[f32]) -> SampleBuffer {
        SampleBuffer {
            channels: 1,
            buffers: vec![SampleBufferMono::from_array(a)],
            samples_count: a.len(),
        }
    }

    pub fn from_vec(a: Vec<f32>) -> SampleBuffer {
        SampleBuffer {
            channels: 1,
            samples_count: a.len(),
            buffers: vec![SampleBufferMono::from_vec(a)],
        }
    }

    pub fn build(&mut self) -> Result<SampleBuffer, Error> {
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

    pub fn fill(&mut self, value: f32) -> usize {
        self.iter_mut().map(|s| *s = value).count()
    }

    pub fn get_mut(&mut self, index: usize) -> Result<&mut f32, Error> {
        let sample = self
            .samples
            .get_mut(index)
            .ok_or(Error::from("Cannot get mutable sample by index"))?;
        Ok(sample)
    }

    pub fn set_at(&mut self, index: usize, value: f32) -> Result<(), Error> {
        let sample_ref = self.get_mut(index)?;
        *sample_ref = value;
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = f32> + '_ {
        self.samples.iter().copied()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut f32> {
        self.samples.iter_mut()
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

    pub fn at(&self, channel: u8, index: usize) -> Result<f32, Error> {
        let sample = self.get_buffer_ref(channel)?.at(index)?;
        Ok(sample)
    }

    pub fn set_at(&mut self, channel: u8, index: usize, value: f32) -> Result<(), Error> {
        self.get_mut_buffer_ref(channel)?.set_at(index, value)?;
        Ok(())
    }

    pub fn add_at(&mut self, channel: u8, index: usize, value: f32) -> Result<(), Error> {
        let sample = self.get_mut_buffer_ref(channel)?.get_mut(index)?;
        *sample += value;
        Ok(())
    }

    pub fn combine(&mut self, buffer: &SampleBuffer) -> Result<(), Error> {
        if self.len() != buffer.len() {
            return Err("Buffers has different length".into());
        }
        self.buffers.iter_mut().enumerate().for_each(|(index, buf)| {
            let self_iter = buf.iter_mut();
            let mut another_iter = buffer.iter(index as u8).unwrap();
            self_iter.for_each(|s| {
                *s += another_iter.next().unwrap();
            });
        });
        Ok(())
    }

    pub fn fill(&mut self, value: f32) {
        for buffer in self.buffers.iter_mut() {
            buffer.fill(value);
        }
    }

    pub fn iter(&self, channel: u8) -> Result<impl Iterator<Item = f32> + '_, Error> {
        let buffer = self.get_buffer_ref(channel)?;
        Ok(buffer.iter())
    }

    pub fn iter_mut(&mut self, channel: u8) -> Result<impl Iterator<Item = &mut f32>, Error> {
        let buffer = self.get_mut_buffer_ref(channel)?;
        Ok(buffer.iter_mut())
    }

    pub fn get_mut_buffer_ref(&mut self, channel: u8) -> Result<&mut SampleBufferMono, Error> {
        let buffer = self
            .buffers
            .get_mut(channel as usize)
            .ok_or(Error::from("Channel index out of buffer size"))?;
        Ok(buffer)
    }

    pub fn get_buffer_ref(&self, channel: u8) -> Result<&SampleBufferMono, Error> {
        let buffer = self
            .buffers
            .get(channel as usize)
            .ok_or(Error::from("Channel index out of buffer size"))?;
        Ok(buffer)
    }
}

impl Default for SampleBufferBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::sample_buffer::{SampleBufferBuilder, SampleBufferMono};

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

    #[test]
    fn test_buffer_set() {
        let mut buffer = SampleBufferMono::from_array(&[0.1, 0.2, 0.95, -0.93, -0.934]);
        buffer.set_at(0, 1.0).unwrap();
        assert_eq!(buffer.at(0).unwrap(), 1.0);
        let mut buffer = SampleBufferBuilder::new()
            .set_channels(2)
            .set_samples(16)
            .build()
            .unwrap();
        let set = buffer.set_at(2, 0, 0.);
        assert!(set.is_err());
        let set = buffer.set_at(0, 0, 0.);
        assert!(set.is_ok());
    }

    #[test]
    fn test_buffer_add() {
        let mut buffer = SampleBufferBuilder::new()
            .set_channels(2)
            .set_samples(16)
            .build()
            .unwrap();
        buffer.add_at(0, 0, 2.).unwrap();
        assert_eq!(buffer.at(0, 0).unwrap(), 2.);
    }

    #[test]
    fn test_buffer_combine() {
        let mut buffer = SampleBufferBuilder::new()
            .set_channels(2)
            .set_samples(16)
            .build()
            .unwrap();
        buffer.fill(1.0);
        let mut buffer2 = SampleBufferBuilder::new()
            .set_channels(2)
            .set_samples(16)
            .build()
            .unwrap();
        buffer2.fill(3.0);
        buffer.combine(&buffer2).unwrap();
        assert_eq!(buffer.at(0, 8).unwrap(), 4.);
    }
}
