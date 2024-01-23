use crate::{
    error::Error,
    iir::coefficients::{build_filter, Coefficients, FilterType},
    utils::sample_buffer::{BufferSettings, SampleBuffer, SampleBufferMono},
};

use super::{Effect, MonoBufferProcessor, State};

#[derive(Clone, Copy)]
struct StatefulCoefficients {
    // filter: FilterType,
    coefficients: Coefficients,
    state: [f32; 3],
}

impl StatefulCoefficients {
    pub fn new(filter: FilterType, sample_rate: f32) -> Self {
        Self {
            // filter,
            coefficients: build_filter(&filter, sample_rate),
            state: [0.0; 3],
        }
    }
}

impl MonoBufferProcessor for StatefulCoefficients {
    fn process(&mut self, buffer: &mut SampleBufferMono) {
        self.coefficients
            .process(buffer.get_slice_mut(), &mut self.state);
    }
}

pub struct Filter {
    coefficients: Vec<StatefulCoefficients>,
    state: State,
}

impl Filter {
    pub fn new(filter: FilterType, settings: &BufferSettings) -> Self {
        Self {
            coefficients: vec![
                StatefulCoefficients::new(filter, settings.sample_rate);
                settings.channels
            ],
            state: State::Enabled,
        }
    }
    fn recalculate_coefficients(&mut self) {}
}

impl Effect for Filter {
    fn process(&mut self, buffer: &mut SampleBuffer) -> Result<(), Error> {
        self.recalculate_coefficients();
        buffer
            .iter_buffers()
            .enumerate()
            .for_each(|(i, buffer)| self.coefficients[i].process(buffer));

        Ok(())
    }

    fn state(&self) -> State {
        self.state
    }

    fn set_state(&mut self, state: State) {
        self.state = state;
    }
}
