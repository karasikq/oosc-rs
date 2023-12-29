use std::sync::{Arc, Mutex, RwLock};

use crate::effects::Effect;
use rayon::prelude::*;

use super::{note::Note, oscillator::Oscillator};
use crate::{
    error::Error,
    utils::sample_buffer::{SampleBuffer, SampleBufferBuilder},
};

pub type LockedOscillator = Arc<RwLock<dyn Oscillator>>;
pub type LockedEffect = Arc<RwLock<dyn Effect + Sync + Send>>;

pub type SyncSynthesizer = Arc<Mutex<Synthesizer>>;

pub struct Synthesizer {
    buffer: SampleBuffer,
    oscillators: Vec<LockedOscillator>,
    effects: Vec<LockedEffect>,
    sample_rate: u32,
}

impl Synthesizer {
    pub fn output(&mut self) -> Result<&SampleBuffer, Error> {
        let buffer = &mut self.buffer;
        let delta_time = 1.0 / self.sample_rate as f32;
        buffer.fill(0.);
        self.oscillators
            .par_iter_mut()
            .try_for_each(|osc| -> Result<(), Error> {
                osc.write().unwrap().evaluate(delta_time)
            })?;
        self.oscillators
            .iter()
            .try_for_each(|osc| -> Result<(), Error> {
                buffer.combine(osc.write().unwrap().get_buffer())
            })?;
        self.effects
            .iter_mut()
            .try_for_each(|effect| -> Result<(), Error> {
                effect.write().unwrap().process(buffer)
            })?;
        Ok(&self.buffer)
    }

    pub fn get_buffer(&self) -> &SampleBuffer {
        &self.buffer
    }

    pub fn release_all(&mut self) {
        self.oscillators.par_iter_mut().for_each(|osc| {
            osc.write().unwrap().release_all();
        })
    }

    pub fn note_on(&mut self, note: Note) -> Result<(), Error> {
        self.oscillators
            .par_iter_mut()
            .try_for_each(|osc| -> Result<(), Error> { osc.write().unwrap().note_on(note) })
    }

    pub fn note_off(&mut self, note: u32) -> Result<(), Error> {
        self.oscillators
            .par_iter_mut()
            .try_for_each(|osc| -> Result<(), Error> { osc.write().unwrap().note_off(note) })
    }

    pub fn get_oscillators<T>(&mut self) -> impl Iterator<Item = LockedOscillator> + '_
    where
        T: Oscillator + 'static,
    {
        self.oscillators.iter_mut().filter_map(|osc| {
            let mut osc_lock = osc.write().unwrap();
            osc_lock
                .as_any_mut()
                .downcast_mut::<T>()
                .map(|_| osc.clone())
        })
    }
}

#[derive(Default)]
pub struct SynthesizerBuilder {
    buffer: Option<SampleBuffer>,
    oscillators: Option<Vec<LockedOscillator>>,
    effects: Option<Vec<LockedEffect>>,
    sample_rate: Option<u32>,
}

impl SynthesizerBuilder {
    pub fn new() -> Self {
        Self {
            buffer: None,
            oscillators: None,
            effects: None,
            sample_rate: None,
        }
    }

    pub fn set_buffer(&mut self, buffer_size: usize) -> Result<&mut Self, Error> {
        self.buffer = Some(
            SampleBufferBuilder::new()
                .set_channels(2)
                .set_samples(buffer_size)
                .build()?,
        );
        Ok(self)
    }

    pub fn add_osc(&mut self, osc: LockedOscillator) -> &mut Self {
        if let Some(vec) = self.oscillators.as_mut() {
            vec.push(osc);
        } else {
            self.oscillators = Some(vec![osc]);
        }
        self
    }

    pub fn empty_osc(&mut self) -> &mut Self {
        self.oscillators = Some(vec![]);
        self
    }

    pub fn add_effect(&mut self, effect: LockedEffect) -> &mut Self {
        if let Some(effects) = self.effects.as_mut() {
            effects.push(effect);
        } else {
            self.effects = Some(vec![effect]);
        }
        self
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) -> &mut Self {
        self.sample_rate = Some(sample_rate);
        self
    }

    pub fn build(&mut self) -> Result<Synthesizer, Error> {
        let buffer = self.buffer.take().ok_or(Error::Specify("buffer size"))?;
        let oscillators = self
            .oscillators
            .take()
            .ok_or(Error::Specify("oscillators"))?;
        let sample_rate = self.sample_rate.ok_or(Error::Specify("sample_rate"))?;
        let effects = self.effects.take().unwrap_or(vec![]);

        Ok(Synthesizer {
            buffer,
            oscillators,
            effects,
            sample_rate,
        })
    }
}
