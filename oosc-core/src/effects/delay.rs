use crate::{
    core::parameter::{Parametr, ValueParametr},
    error::Error,
    utils::{
        interpolation::{interpolate_sample_mut, InterpolateMethod},
        sample_buffer::{BufferSettings, SampleBuffer, SampleBufferBuilder, SampleBufferMono},
    },
};

use super::{Effect, State};

pub struct Delay {
    settings: BufferSettings,
    buffer: SampleBuffer,
    time: Vec<usize>,
    mix: ValueParametr<f32>,
    feedback: ValueParametr<f32>,
    delay: ValueParametr<f32>,
    state: State,
}

impl Delay {
    pub fn new(
        settings: &BufferSettings,
        mix: ValueParametr<f32>,
        feedback: ValueParametr<f32>,
        delay: ValueParametr<f32>,
        state: State,
    ) -> Self {
        let sampled_time = (delay.range().1 * settings.sample_rate).round() as usize;
        let buffer = SampleBufferBuilder::new()
            .set_channels(settings.channels as u32)
            .set_samples(sampled_time)
            .build()
            .unwrap();

        Self {
            settings: *settings,
            buffer,
            time: vec![0; settings.channels],
            mix,
            feedback,
            delay,
            state,
        }
    }

    pub fn default(settings: &BufferSettings) -> Self {
        let mix = ValueParametr::<f32>::new(1.0, (0.0, 1.0));
        let feedback = ValueParametr::<f32>::new(0.7, (0.0, 1.0));
        let delay = ValueParametr::<f32>::new(0.01, (0.0, 0.1));

        Self::new(settings, mix, feedback, delay, State::Enabled)
    }

    fn proccess_channel(
        &mut self,
        buffer: &mut SampleBufferMono,
        index: usize,
    ) -> Result<(), Error> {
        let sample_rate = self.settings.sample_rate;
        let delay = sample_rate * self.delay.get_value();
        let mix = self.mix.get_value();
        let feedback = self.feedback.get_value();
        let table = self.buffer.get_mut_buffer_ref(index as u32)?;

        let len = self.time.len();
        let last_time = self
            .time
            .get_mut(index)
            .ok_or(Error::OutOfRange(index, len))?;

        let len = table.len();
        let len_f32 = len as f32;
        let delay_buffer = table.get_slice_mut();

        buffer.iter_mut().try_for_each(|s| -> Result<(), Error> {
            let current_time = *last_time as f32;
            let dry = *s;
            let index = (current_time - delay + len_f32) % len_f32;
            let out = interpolate_sample_mut(InterpolateMethod::Linear, delay_buffer, index)?;
            *s = dry + mix * (out - dry);
            let index = *last_time % len;
            let ts = delay_buffer.get_mut(index).unwrap();
            *ts = dry + out * feedback;
            *last_time += 1;
            Ok(())
        })?;

        Ok(())
    }
}

impl Effect for Delay {
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
