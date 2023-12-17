use crate::{
    error::Error,
    utils::{
        consts::PI_2M,
        evaluate::Evaluate,
        interpolation::{interpolate_sample, InterpolateMethod},
        sample_buffer::{BufferSettings, SampleBufferMono},
    },
};

use super::waveshape::WaveShape;

pub struct WaveTable {
    buffer: SampleBufferMono,
    chunk_size: usize,
    position: usize,
    interpolation: InterpolateMethod,
}

pub struct WaveTableBuilder {
    buffer: Option<SampleBufferMono>,
    chunk_size: Option<usize>,
    position: Option<usize>,
    interpolation: Option<InterpolateMethod>,
}

impl WaveTableBuilder {
    pub fn new() -> WaveTableBuilder {
        WaveTableBuilder {
            buffer: None,
            chunk_size: None,
            position: None,
            interpolation: None,
        }
    }

    pub fn set_buffer(&mut self, buffer: SampleBufferMono) -> &mut Self {
        self.buffer = Some(buffer);
        self
    }

    pub fn set_chunk_size(&mut self, size: usize) -> &mut Self {
        self.chunk_size = Some(size);
        self
    }

    pub fn set_position(&mut self, position: usize) -> &mut Self {
        self.position = Some(position);
        self
    }

    pub fn set_interpolation(&mut self, interpolation: InterpolateMethod) -> &mut Self {
        self.interpolation = Some(interpolation);
        self
    }

    pub fn build(&mut self) -> Result<WaveTable, Error> {
        let buffer = self.buffer.take().ok_or(Error::Specify("buffer"))?;
        let chunk_size = self.chunk_size.take().ok_or(Error::Specify("chunk size"))?;
        if buffer.len() % chunk_size != 0 {
            return Err(Error::from(
                "Cannot split sample buffer into the same length chunks",
            ));
        }
        let position = self.position.unwrap_or(0);
        let interpolation = self
            .interpolation
            .take()
            .unwrap_or(InterpolateMethod::Floor);
        Ok(WaveTable {
            buffer,
            chunk_size,
            position,
            interpolation,
        })
    }

    pub fn from_array(&mut self, a: &[f32], chunk_size: usize) -> &mut Self {
        let buffer = SampleBufferMono::from(a);
        self.set_buffer(buffer);
        self.set_chunk_size(chunk_size);
        self
    }

    pub fn from_shape(&mut self, shape: WaveShape, chunk_size: usize) -> &mut Self {
        let step = PI_2M / (chunk_size as f32);
        let vec: Vec<f32> = (0..chunk_size)
            .map(|v| shape.evaluate(v as f32 * step).unwrap())
            .collect();
        let buffer = SampleBufferMono::from(vec);
        self.set_buffer(buffer);
        self.set_chunk_size(chunk_size);
        self
    }
}

impl Default for WaveTableBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&BufferSettings> for WaveTable {
    fn from(value: &BufferSettings) -> Self {
        Self {
            buffer: SampleBufferMono::new(value.samples),
            chunk_size: value.samples,
            position: 0,
            interpolation: InterpolateMethod::Linear,
        }
    }
}

impl From<usize> for WaveTable {
    fn from(value: usize) -> Self {
        Self {
            buffer: SampleBufferMono::new(value),
            chunk_size: value,
            position: 0,
            interpolation: InterpolateMethod::Linear,
        }
    }
}

impl WaveTable {
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn chunk_len(&self) -> usize {
        self.chunk_size
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn interpolation(&self) -> InterpolateMethod {
        self.interpolation
    }

    fn set_position(&mut self, position: usize) -> Result<(), Error> {
        if position >= self.chunks() {
            Err("Position should be less than chunks count")?
        } else {
            self.position = position;
            Ok(())
        }
    }

    pub fn chunks(&self) -> usize {
        self.buffer.len() / self.chunk_size
    }

    pub fn get_slice(&self) -> Result<&[f32], Error> {
        Ok(self
            .buffer
            .get_slice()
            .chunks_exact(self.chunk_size)
            .nth(self.position)
            .ok_or(format!(
                "Cannot get {} position of wavetable",
                self.position
            ))?)
    }

    pub fn get_slice_mut(&mut self) -> Result<&mut [f32], Error> {
        Ok(self
            .buffer
            .get_slice_mut()
            .chunks_exact_mut(self.chunk_size)
            .nth(self.position)
            .ok_or(format!(
                "Cannot get {} position of wavetable",
                self.position
            ))?)
    }

    pub fn load_new(&mut self, samples: Vec<f32>, chunk_size: usize) {
        self.buffer = SampleBufferMono::from(samples);
        self.chunk_size = chunk_size;
        self.position = 0;
    }
}

impl Evaluate<f32> for WaveTable {
    fn evaluate(&self, t: f32) -> Result<f32, Error> {
        let chunk_slice = self.get_slice()?;
        let chunk = (self.chunk_size - 1) as f32;
        let index = chunk * (t % PI_2M / PI_2M);
        interpolate_sample(self.interpolation, chunk_slice, index)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::consts::PI;
    use crate::utils::evaluate::Evaluate;
    use crate::utils::interpolation::InterpolateMethod;
    use crate::utils::sample_buffer::SampleBufferMono;
    use crate::{
        core::{waveshape::WaveShape, wavetable::WaveTableBuilder},
        utils::consts::PI_2M,
    };
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_wavetable_shape() {
        let table = WaveTableBuilder::new()
            .from_shape(WaveShape::Sin, 2048)
            .build()
            .unwrap();
        assert_approx_eq!(table.evaluate(0.).unwrap(), 0.);
        assert_approx_eq!(table.evaluate(PI_2M).unwrap(), 0.);
        assert_approx_eq!(table.evaluate(PI / 2.).unwrap(), 1., 1e-3);
        assert_approx_eq!(table.evaluate(PI / 3.).unwrap(), 0.866, 1e-3);
    }

    #[test]
    fn test_wavetable_array() {
        let samples = [0.1, -0.2, 0.6, 0.96, 0.3, 0.55];
        let _ = WaveTableBuilder::new()
            .from_array(&samples, 2)
            .build()
            .unwrap();
        let _ = WaveTableBuilder::new()
            .from_array(&samples, 3)
            .build()
            .unwrap();
        let table = WaveTableBuilder::new().from_array(&samples, 4).build();
        assert!(table.is_err());
    }

    #[test]
    fn test_wavetable_position() {
        let samples = [0.1, -0.2, 0.6, 0.96, 0.3, 0.55];
        let mut table = WaveTableBuilder::new()
            .from_array(&samples, 2)
            .set_interpolation(InterpolateMethod::Linear)
            .build()
            .unwrap();
        table.set_position(0).unwrap();
        assert_approx_eq!(table.evaluate(PI).unwrap(), -0.05);
        table.set_position(1).unwrap();
        assert_approx_eq!(table.evaluate(PI).unwrap(), 0.78);
        assert!(table.set_position(3).is_err());
    }

    #[test]
    fn test_wavetable_builder() {
        let samples = [0.1, -0.2, 0.6, 0.96, 0.3, 0.55];
        WaveTableBuilder::new()
            .set_buffer(SampleBufferMono::from(samples))
            .set_chunk_size(2)
            .set_position(0)
            .build()
            .unwrap();
    }
}
