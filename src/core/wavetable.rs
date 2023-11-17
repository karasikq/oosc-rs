use crate::{
    error::Error,
    utils::{
        consts::PI_2M,
        evaluate::Evaluate,
        sample_buffer::{SampleBuffer, SampleBufferBuilder},
    },
};

use super::waveshape::WaveShape;

pub struct WaveTable {
    buffer: SampleBuffer,
    chunk_size: usize,
    chunks: usize,
    position: usize,
}

pub struct WaveTableBuilder {
    buffer: Option<SampleBuffer>,
    chunk_size: Option<usize>,
    position: Option<usize>,
}

impl WaveTableBuilder {
    pub fn new() -> WaveTableBuilder {
        WaveTableBuilder {
            buffer: None,
            chunk_size: None,
            position: None,
        }
    }

    pub fn set_buffer(&mut self, buffer: SampleBuffer) {
        self.buffer = Some(buffer)
    }

    pub fn set_chunk_size(&mut self, size: usize) {
        self.chunk_size = Some(size)
    }

    pub fn set_position(&mut self, position: usize) {
        self.position = Some(position)
    }

    pub fn build(self) -> Result<WaveTable, Error> {
        let buffer = self.buffer.ok_or(Error::Specify("buffer"))?;
        let chunk_size = self.chunk_size.ok_or(Error::Specify("chunk size"))?;
        if buffer.len() % chunk_size != 0 {
            return Err(Error::from(
                "Cannot split sample buffer into the same length chunks",
            ));
        }
        let chunks = buffer.len() / chunk_size;
        let position = self.position.unwrap_or(0);
        Ok(WaveTable {
            buffer,
            chunk_size,
            chunks,
            position,
        })
    }

    pub fn from_array(mut self, a: &[f32], chunk_size: usize) -> Result<WaveTable, Error> {
        let buffer = SampleBufferBuilder::new().from_array(a);
        self.set_buffer(buffer);
        self.set_chunk_size(chunk_size);
        self.build()
    }

    pub fn from_shape(mut self, shape: WaveShape, chunk_size: usize) -> Result<WaveTable, Error> {
        let step = PI_2M / (chunk_size as f32);
        let vec: Vec<f32> = (0..chunk_size)
            .map(|v| shape.evaluate(v as f32 * step).unwrap())
            .collect();
        let buffer = SampleBufferBuilder::new().from_vec(vec);
        self.set_buffer(buffer);
        self.set_chunk_size(chunk_size);
        self.build()
    }
}

impl Default for WaveTableBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl WaveTable {
    fn sample_at(&self, index: usize) -> Result<f32, Error> {
        self.buffer.at(index, 0)
    }

    fn set_position(&mut self, position: usize) -> Result<(), Error> {
        if position >= self.chunks {
            Err("Position should be less than chunks count")?
        } else {
            self.position = position;
            Ok(())
        }
    }
}

impl Evaluate for WaveTable {
    fn evaluate(&self, t: f32) -> Result<f32, Error> {
        let index = ((self.chunk_size as f32 * (t % PI_2M / PI_2M)).ceil() % self.chunk_size as f32)
            as usize;
        self.sample_at(self.position * self.chunk_size + index)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::consts::PI;
    use crate::utils::evaluate::Evaluate;
    use crate::{
        core::{waveshape::WaveShape, wavetable::WaveTableBuilder},
        utils::consts::PI_2M,
    };
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_wavetable_shape() {
        let table = WaveTableBuilder::new()
            .from_shape(WaveShape::Sin, 2048)
            .unwrap();
        assert_approx_eq!(table.evaluate(0.).unwrap(), 0.);
        assert_approx_eq!(table.evaluate(PI_2M).unwrap(), 0.);
        assert_approx_eq!(table.evaluate(PI / 2.).unwrap(), 1.);
        assert_approx_eq!(table.evaluate(PI / 3.).unwrap(), 0.866, 10e-3);
    }

    #[test]
    fn test_wavetable_array() {
        let samples = [0.1, -0.2, 0.6, 0.96, 0.3, 0.55];
        let table = WaveTableBuilder::new().from_array(&samples, 2);
        assert!(table.is_ok());
        let table = WaveTableBuilder::new().from_array(&samples, 3);
        assert!(table.is_ok());
        let table = WaveTableBuilder::new().from_array(&samples, 4);
        assert!(table.is_err());
    }

    #[test]
    fn test_wavetable_position() {
        let samples = [0.1, -0.2, 0.6, 0.96, 0.3, 0.55];
        let mut table = WaveTableBuilder::new().from_array(&samples, 3).unwrap();
        assert!(table.set_position(1).is_ok());
        assert!(table.set_position(2).is_err());
    }
}
