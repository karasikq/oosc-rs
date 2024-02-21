use crate::{
    core::{
        parameter::{
            NamedParameter, NamedParametersContainer, Parameter, SharedParameter, ValueParameter,
            VolumeParameter,
        },
        waveshape::WaveShape,
    },
    error::Error,
    utils::{
        consts::{PI_2, PI_2M},
        evaluate::Evaluate,
        interpolation::{interpolate_sample_mut, InterpolateMethod},
        make_shared,
        sample_buffer::{BufferSettings, SampleBuffer, SampleBufferBuilder, SampleBufferMono},
        Shared,
    },
};

use super::{Effect, State};

pub struct Chorus {
    settings: BufferSettings,
    buffer: SampleBuffer,
    time: Vec<usize>,
    depth: Shared<VolumeParameter>,
    rate: SharedParameter<f32>,
    phase: SharedParameter<f32>,
    lfo: WaveShape,
    width: SharedParameter<f32>,
    delay: SharedParameter<f32>,
    state: State,
    parameters_f32: Vec<NamedParameter<f32>>,
}

impl Chorus {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        settings: &BufferSettings,
        depth: VolumeParameter,
        rate: ValueParameter<f32>,
        phase: ValueParameter<f32>,
        lfo: WaveShape,
        width: ValueParameter<f32>,
        delay: ValueParameter<f32>,
        state: State,
    ) -> Self {
        let sampled_time =
            ((width.range().1 + delay.range().1) * settings.sample_rate).round() as usize;
        let buffer = SampleBufferBuilder::new()
            .set_channels(settings.channels as u32)
            .set_samples(sampled_time)
            .build()
            .unwrap();
        let depth = make_shared(depth);
        let rate = make_shared(rate);
        let phase = make_shared(phase);
        let width = make_shared(width);
        let delay = make_shared(delay);

        let parameters_f32 = vec![
            NamedParameter::new(depth.clone(), "Depth"),
            NamedParameter::new(rate.clone(), "Rate"),
            NamedParameter::new(phase.clone(), "Phase"),
            NamedParameter::new(width.clone(), "Width"),
            NamedParameter::new(delay.clone(), "Delay"),
        ];

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
            parameters_f32,
        }
    }

    pub fn default(settings: &BufferSettings) -> Self {
        let depth = VolumeParameter::new(ValueParameter::<f32>::new(-3.0, (-96.0, 3.0)));
        let rate = ValueParameter::<f32>::new(0.2, (0.01, 20.0));
        let phase = ValueParameter::<f32>::new(PI_2, (0.0, PI_2M));
        let lfo = WaveShape::Triangle;
        let width = ValueParameter::<f32>::new(0.05, (0.0, 0.1));
        let delay = ValueParameter::<f32>::new(0.05, (0.0, 0.1));

        Self::new(
            settings,
            depth,
            rate,
            phase,
            lfo,
            width,
            delay,
            State::Enabled,
        )
    }

    fn proccess_channel(
        &mut self,
        buffer: &mut SampleBufferMono,
        index: usize,
        size: usize,
    ) -> Result<(), Error> {
        let sample_rate = self.settings.sample_rate;
        let rate = self.rate.read().unwrap().get_value();
        let delay = self.delay.read().unwrap().get_value();
        let width = self.width.read().unwrap().get_value();
        let depth = self.depth.read().unwrap().linear;
        let table = self.buffer.get_mut_buffer_ref(index as u32).unwrap();
        let phase = self.phase.read().unwrap().get_value() * index as f32;

        let len = self.time.len();
        let last_time = self
            .time
            .get_mut(index)
            .ok_or(Error::OutOfRange(index, len))?;

        let len = table.len();
        let len_f32 = len as f32;
        let delay_buffer = table.get_slice_mut();

        buffer
            .iter_mut()
            .take(size)
            .try_for_each(|s| -> Result<(), Error> {
                let dry = *s;
                let current_time = *last_time as f32;
                let delay_time = sample_rate
                    * (delay
                        + width
                            * self
                                .lfo
                                .evaluate(current_time / sample_rate * rate + phase)?);
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
    fn process(&mut self, size: usize, buffer: &mut SampleBuffer) -> Result<(), Error> {
        buffer
            .iter_buffers()
            .enumerate()
            .try_for_each(|(i, buffer)| self.proccess_channel(buffer, i, size))
    }

    fn state(&self) -> State {
        self.state
    }

    fn set_state(&mut self, state: State) {
        self.state = state;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn parameters(&mut self) -> Option<&mut dyn NamedParametersContainer> {
        Some(self)
    }
}

impl NamedParametersContainer for Chorus {
    fn name(&self) -> Option<&'static str> {
        Some("Chorus")
    }

    fn parameters_f32(&self) -> Option<&[NamedParameter<f32>]> {
        Some(&self.parameters_f32)
    }
}
