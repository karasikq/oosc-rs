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
    pub fn new() -> Self {
        Self {
            buffer: None,
            chunk_size: None,
            position: None,
        }
    }

    pub fn from_array(self, a: &[f32], chunk_size: usize) -> Result<WaveTable, Error> {
        let buffer = SampleBufferBuilder::new().from_array(a);
        let chunks = buffer.len() / chunk_size;
        Ok(WaveTable {
            buffer,
            chunk_size,
            chunks,
            position: 0,
        })
    }

    pub fn from_shape(shape: WaveShape, chunk_size: usize) -> Result<WaveTable, Error> {
        let step = PI_2M / (chunk_size as f32);
        let vec: Vec<f32> = (0..chunk_size)
            .map(|v| shape.evaluate(v as f32 * step).unwrap())
            .collect();
        let buffer = SampleBufferBuilder::new().from_vec(vec);
        let chunks = buffer.len() / chunk_size;
        Ok(WaveTable {
            buffer,
            chunk_size,
            chunks,
            position: 0,
        })
    }
}

impl WaveTable {
    fn sample_at(&self, index: usize) -> Result<f32, Error> {
        self.buffer.at(index, 0)
    }
}

impl Evaluate for WaveTable {
    fn evaluate(&self, t: f32) -> Result<f32, Error> {
        let index = ((self.chunk_size as f32 * (t % PI_2M / PI_2M)).ceil() % self.chunk_size as f32)
            as usize;
        println!("{}", index);
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
        let table = WaveTableBuilder::from_shape(WaveShape::Sin, 2048).unwrap();
        assert_approx_eq!(table.evaluate(PI_2M).unwrap(), 0.);
        assert_approx_eq!(table.evaluate(PI / 2.).unwrap(), 1.);
        assert_approx_eq!(table.evaluate(PI / 3.).unwrap(), 0.866, 10e-3);
    }
}
