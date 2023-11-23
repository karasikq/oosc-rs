use crate::{
    error::Error,
    utils::{
        consts::PI_2M,
        evaluate::Evaluate,
        interpolation::{interpolate_lagrange, interpolate_linear, InterpolateMethod},
        sample_buffer::{SampleBuffer, SampleBufferBuilder},
    },
};

use super::waveshape::WaveShape;

pub struct WaveTable {
    buffer: SampleBuffer,
    chunk_size: usize,
    chunks: usize,
    position: usize,
    interpolation: InterpolateMethod,
}

pub struct WaveTableBuilder {
    buffer: Option<SampleBuffer>,
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

    pub fn set_buffer(&mut self, buffer: SampleBuffer) -> &mut Self {
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
        let chunks = buffer.len() / chunk_size;
        let position = self.position.unwrap_or(0);
        let interpolation = self.interpolation.take().unwrap_or(InterpolateMethod::Ceil);
        Ok(WaveTable {
            buffer,
            chunk_size,
            chunks,
            position,
            interpolation,
        })
    }

    pub fn from_array(&mut self, a: &[f32], chunk_size: usize) -> &mut Self {
        let buffer = SampleBufferBuilder::from_array(a);
        self.set_buffer(buffer);
        self.set_chunk_size(chunk_size);
        self
    }

    pub fn from_shape(&mut self, shape: WaveShape, chunk_size: usize) -> &mut Self {
        let step = PI_2M / (chunk_size as f32);
        let vec: Vec<f32> = (0..chunk_size)
            .map(|v| shape.evaluate(v as f32 * step).unwrap())
            .collect();
        let buffer = SampleBufferBuilder::from_vec(vec);
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

impl WaveTable {
    fn sample_at(&self, index: usize) -> Result<f32, Error> {
        self.buffer.at(0, index)
    }

    fn set_position(&mut self, position: usize) -> Result<(), Error> {
        if position >= self.chunks {
            Err("Position should be less than chunks count")?
        } else {
            self.position = position;
            Ok(())
        }
    }

    pub fn get_samples_ranged(
        &self,
        start: i32,
        count: i32,
    ) -> impl Iterator<Item = cgmath::Vector2<f32>> + '_ {
        let end = start + count;
        let chunk = self.chunk_size as i32;
        (start..end).map(move |index| {
            let mut end = index;
            if index < 0 {
                end = chunk + index;
            }
            if index >= chunk {
                end = index - chunk
            }
            let sample = self.sample_at(end as usize).unwrap();
            cgmath::Vector2::<f32>::new(end as f32, sample)
        })
    }
}

impl Evaluate<f32> for WaveTable {
    fn evaluate(&self, t: f32) -> Result<f32, Error> {
        let chunk = self.chunk_size as f32;
        let index = (chunk * (t % PI_2M / PI_2M)) % chunk;
        let fraction = index % 1.0;
        let index_ceil = index.ceil();
        if fraction <= 10e-6 {
            return self.sample_at(index_ceil as usize);
        }
        Ok(match self.interpolation {
            InterpolateMethod::Ceil => self.sample_at(index_ceil as usize)?,
            InterpolateMethod::Linear => {
                let mut samples = self.get_samples_ranged(index_ceil as i32, 2);
                let s1 = samples.next().unwrap();
                let s2 = samples.next().unwrap();
                interpolate_linear(s1.y, s2.y, fraction)
            }
            InterpolateMethod::LaGrange => {
                let left_index = (index.ceil() - 1.) as i32;
                let vec = self.get_samples_ranged(left_index, 4).collect();
                interpolate_lagrange(vec, index)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::consts::PI;
    use crate::utils::evaluate::Evaluate;
    use crate::utils::interpolation::InterpolateMethod;
    use crate::utils::sample_buffer::SampleBufferBuilder;
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
        assert_approx_eq!(table.evaluate(PI / 2.).unwrap(), 1.);
        assert_approx_eq!(table.evaluate(PI / 3.).unwrap(), 0.866, 10e-3);
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
            .from_array(&samples, 3)
            .build()
            .unwrap();
        assert!(table.set_position(1).is_ok());
        assert!(table.set_position(2).is_err());
    }

    #[test]
    fn test_wavetable_builder() {
        let samples = [0.1, -0.2, 0.6, 0.96, 0.3, 0.55];
        WaveTableBuilder::new()
            .set_buffer(SampleBufferBuilder::from_array(&samples))
            .set_chunk_size(2)
            .set_position(0)
            .build()
            .unwrap();
    }

    #[test]
    fn test_wavetable_get_ranged() {
        let size = 2048;
        let step = 1.0 / size as f32;
        let table = WaveTableBuilder::new()
            .from_shape(WaveShape::Sin, 2048)
            .set_interpolation(InterpolateMethod::LaGrange)
            .build()
            .unwrap();
        let mut samples = table.get_samples_ranged(0, 3);
        assert_approx_eq!(samples.next().unwrap().y, (0. * step).sin(), 10e-3);
        assert_approx_eq!(samples.next().unwrap().y, (1. * step).sin(), 10e-3);
        assert_approx_eq!(samples.next().unwrap().y, (2. * step).sin(), 10e-3);
        assert!(samples.next().is_none());
    }
}
