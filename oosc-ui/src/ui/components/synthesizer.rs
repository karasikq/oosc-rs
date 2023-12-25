use oosc_core::core::{oscillator::WavetableOscillator, synthesizer::Synthesizer};
use ratatui::{prelude::*, widgets::*};

use super::{oscillator::OscillatorComponent, Component, Focus, FocusableComponent};

pub enum Action {
    SelectOscillator(u8),
    SelectEffectsBlock,
}

pub struct SynthesizerComponent {
    pub oscillators: Vec<OscillatorComponent>,
    pub rect: Rect,
}

impl SynthesizerComponent {
    pub fn new(synthesizer: &mut Synthesizer, rect: Rect) -> Self {
        let oscillators: Vec<OscillatorComponent> = synthesizer
            .get_oscillators::<WavetableOscillator>()
            .map(|osc| OscillatorComponent::new(osc))
            .collect();
        let mut component = Self { oscillators, rect };
        component.resize(rect).unwrap();
        component
    }
}

impl FocusableComponent for SynthesizerComponent {}

impl Component for SynthesizerComponent {
    type Action = Action;

    fn draw(
        &mut self,
        f: &mut ratatui::Frame<'_>,
        rect: ratatui::prelude::Rect,
    ) -> anyhow::Result<()> {
        let border = Block::default()
            .borders(Borders::ALL)
            .title("oosc")
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center);
        border.render(self.rect, f.buffer_mut());
        self.oscillators.iter_mut().for_each(|osc| {
            osc.draw(f, rect).unwrap();
        });
        Ok(())
    }

    fn resize(&mut self, rect: Rect) -> anyhow::Result<()> {
        let oscillators = &mut self.oscillators;
        let size = 100 / oscillators.len();
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                std::iter::repeat_with(|| Constraint::Percentage(size as u16))
                    .take(oscillators.len())
                    .collect::<Vec<_>>(),
            )
            .margin(1)
            .split(rect);
        oscillators
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, osc)| osc.resize(*layout.get(i).unwrap()))
    }

    fn handle_key_events(&mut self, key: crossterm::event::KeyEvent) -> anyhow::Result<()> {
        self.oscillators
            .iter_mut()
            .try_for_each(|osc| osc.handle_key_events(key))
    }
}

impl Focus for SynthesizerComponent {
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
