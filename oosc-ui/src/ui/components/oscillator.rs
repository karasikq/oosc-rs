use std::any::Any;

use oosc_core::core::oscillator::WavetableOscillator;

use super::{Component, Focus, FocusableComponent};

pub struct OscillatorComponent {
    pub oscillator: Box<WavetableOscillator>,
}

impl FocusableComponent for OscillatorComponent {}

impl Component for OscillatorComponent {
    fn draw(&mut self, f: &mut ratatui::Frame<'_>, rect: ratatui::prelude::Rect) -> anyhow::Result<()> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
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
