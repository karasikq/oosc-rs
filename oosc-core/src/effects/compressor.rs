use std::cell::RefCell;
use std::rc::Rc;

use crate::effects::Effect;
use crate::utils::convert::voltage_to_linear;
use crate::{
    core::parametrs::{ExponentialTimeParametr, Parametr, ValueParametr, VolumeParametr},
    error::Error,
    utils::sample_buffer::SampleBuffer,
};

use super::sample_detector::{SampleDetector, TimeParametr};
use super::SampleProcessor;

pub enum KneeType {
    Soft(VolumeParametr),
    Hard,
}

pub struct Compressor {
    threshold: VolumeParametr,
    ratio: ValueParametr<f32>,
    knee_type: KneeType,
    attack: TimeParametr,
    release: TimeParametr,
    detectors: Vec<SampleDetector>,
}

impl Compressor {
    pub fn new(
        threshold: VolumeParametr,
        ratio: ValueParametr<f32>,
        knee_type: KneeType,
        attack: ExponentialTimeParametr,
        release: ExponentialTimeParametr,
        channels: usize,
    ) -> Self {
        let attack = Rc::new(RefCell::new(attack));
        let release = Rc::new(RefCell::new(release));
        let detectors = (0..channels)
            .map(|_| SampleDetector::new(attack.clone(), release.clone()))
            .collect();

        Self {
            threshold,
            ratio,
            knee_type,
            attack,
            release,
            detectors,
        }
    }

    pub fn default(channels: usize, sample_rate: f32) -> Self {
        let threshold = VolumeParametr::new(ValueParametr::new(-3.0, (-96.0, 0.0)));
        let ratio = ValueParametr::new(50.0, (1.0, 100.0));
        let knee_type = KneeType::Soft(VolumeParametr::new(ValueParametr::new(12.0, (0.0, 36.0))));
        let attack =
            ExponentialTimeParametr::new(ValueParametr::new(0.005, (0.001, 0.5)), sample_rate);
        let release =
            ExponentialTimeParametr::new(ValueParametr::new(0.5, (0.001, 5.0)), sample_rate);

        Self::new(threshold, ratio, knee_type, attack, release, channels)
    }

    fn proccess_sample(&mut self, sample: &mut f32, processor: usize) {
        let detected = self.detectors.get_mut(processor).unwrap().process(*sample);
        let threshold = self.threshold.get_value();
        let ratio = self.ratio.get_value();
        let output = match &self.knee_type {
            KneeType::Soft(width) => {
                let width = width.get_value();
                let region = 2.0 * (detected - threshold);
                if region < -width {
                    detected
                } else if region.abs() <= width {
                    detected
                        + (((1.0 / ratio) - 1.0) * (detected - threshold + width * 0.5).powi(2)
                            / (2.0 * width))
                } else {
                    threshold + (detected - threshold) / ratio
                }
            }
            KneeType::Hard => {
                if detected <= threshold {
                    detected
                } else {
                    threshold + (detected - threshold) / ratio
                }
            }
        };
        *sample = voltage_to_linear(output - detected);
    }
}

impl Effect for Compressor {
    fn process(&mut self, buffer: &mut SampleBuffer) -> Result<(), Error> {
        buffer.iter_buffers().enumerate().for_each(|(i, buffer)| {
            buffer
                .iter_mut()
                .for_each(|sample| self.proccess_sample(sample, i));
        });
        Ok(())
    }
}
