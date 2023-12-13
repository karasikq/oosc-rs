use oosc_core::core::{oscillator::WavetableOscillator, synthesizer::LockedOscillator};
use ratatui::{prelude::*, widgets::*};

use crate::ui::widgets::oscillator::OscillatorWidget;

use super::{Component, Focus, FocusableComponent};

pub enum Action {
    EnterFullscreen,
    MoveNextControl,
    MovePreviousControl,
    ExitFullscreen,
}

pub struct OscillatorComponent {
    pub oscillator: LockedOscillator,
    pub rect: Option<Rect>,
}

impl OscillatorComponent {
    pub fn new(oscillator: LockedOscillator) -> Self {
        Self {
            oscillator,
            rect: None,
        }
    }
}

impl FocusableComponent for OscillatorComponent {}

impl Component for OscillatorComponent {
    type Action = Action;

    fn draw(
        &mut self,
        f: &mut ratatui::Frame<'_>,
        rect: ratatui::prelude::Rect,
    ) -> anyhow::Result<()> {
        let mut osc = self.oscillator.write().unwrap();
        let osc = osc
            .as_any_mut()
            .downcast_mut::<WavetableOscillator>()
            .unwrap();
        let widget = OscillatorWidget { oscillator: osc };
        widget.render(self.rect.unwrap(), f.buffer_mut());
        Ok(())
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
