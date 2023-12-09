use std::sync::{Arc, Mutex};

use crate::effects::{amplifier::Amplifier, Effect};
use rayon::prelude::*;

use super::{
    note::Note,
    oscillator::Oscillator,
    parametrs::{PanParametr, ValueParametr, VolumeParametr},
};
use crate::{
    error::Error,
    utils::sample_buffer::{SampleBuffer, SampleBufferBuilder},
};

type Osc = Box<dyn Oscillator>;
type SynEffect = Box<dyn Effect + Sync + Send>;

pub type SyncSynthesizer = Arc<Mutex<Synthesizer>>;

pub struct Synthesizer {
    buffer: SampleBuffer,
    oscillators: Vec<Osc>,
    effects: Vec<SynEffect>,
    sample_rate: u32,
}

impl Synthesizer {
    pub fn output(&mut self) -> Result<&SampleBuffer, Error> {
        let buffer = &mut self.buffer;
        let delta_time = 1.0 / self.sample_rate as f32;
        buffer.fill(0.);
        self.oscillators
            .par_iter_mut()
            .try_for_each(|osc| -> Result<(), Error> { osc.evaluate(delta_time) })?;
        self.oscillators
            .iter()
            .try_for_each(|osc| -> Result<(), Error> { buffer.combine(osc.get_buffer()) })?;
        self.effects
            .iter_mut()
            .try_for_each(|effect| -> Result<(), Error> { effect.process(buffer) })?;
        Ok(&self.buffer)
    }

    pub fn get_buffer(&self) -> &SampleBuffer {
        &self.buffer
    }

    pub fn release_all(&mut self) {
        self.oscillators.par_iter_mut().for_each(|osc| {
            osc.release_all();
        })
    }

    pub fn note_on(&mut self, note: Note) -> Result<(), Error> {
        self.oscillators
            .par_iter_mut()
            .try_for_each(|osc| -> Result<(), Error> { osc.note_on(note) })
    }

    pub fn note_off(&mut self, note: u32) -> Result<(), Error> {
        self.oscillators
            .par_iter_mut()
            .try_for_each(|osc| -> Result<(), Error> { osc.note_off(note) })
    }

    pub fn get_oscillators<'a, T>(&'a mut self) -> impl Iterator<Item = &'a mut T>
    where
        T: Oscillator + 'a + 'static,
    {
        self.oscillators
            .iter_mut()
            .filter_map(|osc| osc.as_any_mut().downcast_mut::<T>())
    }
}

#[derive(Default)]
pub struct SynthesizerBuilder {
    buffer: Option<SampleBuffer>,
    oscillators: Option<Vec<Osc>>,
    effects: Option<Vec<SynEffect>>,
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

    pub fn add_osc(&mut self, osc: Osc) -> &mut Self {
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

    pub fn add_effect(&mut self, effect: SynEffect) -> &mut Self {
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
        let amplifier = Amplifier::new(
            VolumeParametr::from(ValueParametr::new(-3.0, (-60.0, 3.0))),
            PanParametr::from(ValueParametr::new(0.0, (-1.0, 1.0))),
        );
        self.add_effect(Box::new(amplifier));
        let effects = self.effects.take().unwrap();

        Ok(Synthesizer {
            buffer,
            oscillators,
            effects,
            sample_rate,
        })
    }
}
