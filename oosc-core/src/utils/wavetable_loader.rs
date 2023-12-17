use crate::{core::wavetable::WaveTable, error::Error};

pub fn load_wavetable<P: AsRef<std::path::Path>>(
    path: P,
    table: &mut WaveTable,
) -> Result<(), Error> {
    let mut reader = hound::WavReader::open(path).unwrap();
    let samples: Vec<f32> = reader.samples::<f32>().filter_map(|s| s.ok()).collect();
    table.load_new(samples, 2048);
    Ok(())
}
