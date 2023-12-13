use crate::{
    core::{
        parametrs::{Parametr, ValueParametr, VolumeParametr},
        waveshape::WaveShape,
    },
    error::Error,
    utils::{
        consts::{PI_2M, PI_2},
        evaluate::Evaluate,
        interpolation::{interpolate_sample_mut, InterpolateMethod},
        sample_buffer::{BufferSettings, SampleBuffer, SampleBufferBuilder, SampleBufferMono},
    },
};

use super::{Effect, State};

pub struct Chorus {
    settings: BufferSettings,
    buffer: SampleBuffer,
    time: Vec<usize>,
    depth: VolumeParametr,
    rate: ValueParametr<f32>,
    phase: ValueParametr<f32>,
    lfo: WaveShape,
    width: ValueParametr<f32>,
    delay: ValueParametr<f32>,
    state: State,
}

impl Chorus {
    pub fn new(
        settings: &BufferSettings,
        depth: VolumeParametr,
        rate: ValueParametr<f32>,
        phase: ValueParametr<f32>,
        lfo: WaveShape,
        width: ValueParametr<f32>,
        delay: ValueParametr<f32>,
        state: State,
    ) -> Self {
        let sampled_time =
            ((width.range().1 + delay.range().1) * settings.sample_rate).round() as usize;
        let buffer = SampleBufferBuilder::new()
            .set_channels(settings.channels as u32)
            .set_samples(sampled_time)
            .build()
            .unwrap();

        Self {
            settings: *settings,
            buffer,
            time: vec![0; settings.channels],
            depth,
            rate,
            phase,
            lfo,
            width,
            delay,
            state,
        }
    }

    pub fn default(settings: &BufferSettings) -> Self {
        let depth = VolumeParametr::new(ValueParametr::<f32>::new(-3.0, (-96.0, 3.0)));
        let rate = ValueParametr::<f32>::new(0.2, (0.01, 20.0));
        let phase = ValueParametr::<f32>::new(PI_2, (0.0, PI_2M));
        let lfo = WaveShape::Triangle;
        let width = ValueParametr::<f32>::new(0.05, (0.0, 0.1));
        let delay = ValueParametr::<f32>::new(0.05, (0.0, 0.1));

        Self::new(settings, depth, rate, phase, lfo, width, delay, State::Enabled)
    }

    fn proccess_channel(
        &mut self,
        buffer: &mut SampleBufferMono,
        index: usize,
    ) -> Result<(), Error> {
        let sample_rate = self.settings.sample_rate;
        let rate = self.rate.get_value();
        let delay = self.delay.get_value();
        let width = self.width.get_value();
        let depth = self.depth.linear;
        let table = self.buffer.get_mut_buffer_ref(index as u32).unwrap();
        let phase = self.phase.get_value() * index as f32;

        let len = self.time.len();
        let last_time = self
            .time
            .get_mut(index)
            .ok_or(Error::OutOfRange(index, len))?;

        let len = table.len();
        let len_f32 = len as f32;
        let delay_buffer = table.get_slice_mut();

        buffer.iter_mut().try_for_each(|s| -> Result<(), Error> {
            let dry = *s;
            let current_time = *last_time as f32;
            let delay_time = sample_rate
                * (delay + width * self.lfo.evaluate(current_time / sample_rate * rate + phase)?);
            let index = (current_time - delay_time + len_f32) % len_f32;
            let out = interpolate_sample_mut(InterpolateMethod::Linear, delay_buffer, index)?;
            *s = dry + depth * (out - dry);
            let index = *last_time % len;
            let ts = delay_buffer.get_mut(index).unwrap();
            *ts = dry;
            *last_time += 1;
            Ok(())
        })
    }
}

impl Effect for Chorus {
    fn process(&mut self, buffer: &mut SampleBuffer) -> Result<(), Error> {
        buffer
            .iter_buffers()
            .enumerate()
            .try_for_each(|(i, buffer)| self.proccess_channel(buffer, i))
    }

    fn state(&self) -> State {
        self.state
    }

    fn set_state(&mut self, state: State) {
        self.state = state;
    }
}
