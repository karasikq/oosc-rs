use oosc_core::core::{oscillator::WavetableOscillator, synthesizer::SyncSynthesizer};

use super::{Component, Focus, FocusableComponent};

pub enum Action {
    EnterFullscreen,
    MoveNextControl,
    MovePreviousControl,
    ExitFullscreen,
}

pub struct OscillatorComponent {
    pub synthesizer: SyncSynthesizer,
    pub oscillator_id: usize,
}

impl FocusableComponent for OscillatorComponent {}

impl Component for OscillatorComponent {
    type Action = Action;

    fn draw(&mut self, f: &mut ratatui::Frame<'_>, rect: ratatui::prelude::Rect) -> anyhow::Result<()> {
        todo!()
    }
}

impl Focus for OscillatorComponent {
    fn focus(&mut self) {
        todo!()
    }

    fn unfocus(&mut self) {
        todo!()
    }

    fn is_focused(&self) -> bool {
        todo!()
    }
}
