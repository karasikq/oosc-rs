use crossterm::event::KeyCode;
use oosc_core::core::{oscillator::WavetableOscillator, synthesizer::Synthesizer};
use ratatui::prelude::*;

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
            .enumerate()
            .map(|(i, osc)| {
                let map = KeyCode::Char(char::from_digit(i as u32, 10).unwrap());
                OscillatorComponent::new(osc, map)
            })
            .collect();
        let mut component = Self { oscillators, rect };
        component.resize(rect).unwrap();
        component
    }

    fn unfocus_all(&mut self) {
        self.oscillators.iter_mut().for_each(|osc| osc.unfocus());
    }

    fn is_any_children_focused(&mut self) -> bool {
        self.oscillators.iter().any(|osc| osc.is_focused())
    }
}

impl FocusableComponent for SynthesizerComponent {}

impl Component for SynthesizerComponent {
    fn draw(
        &mut self,
        f: &mut ratatui::Frame<'_>,
        rect: ratatui::prelude::Rect,
    ) -> anyhow::Result<()> {
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
            .split(rect);
        oscillators
            .iter_mut()
            .enumerate()
            .try_for_each(|(i, osc)| osc.resize(*layout.get(i).unwrap()))
    }

    fn handle_key_events(&mut self, key: crossterm::event::KeyEvent) -> anyhow::Result<()> {
        self.oscillators
            .iter_mut()
            .try_for_each(|osc| osc.handle_key_events(key))?;
        if !self.is_any_children_focused() {
            match key.code {
                KeyCode::Char('z') => {
                    self.unfocus_all();
                    self.oscillators.get_mut(0).unwrap().focus();
                }
                KeyCode::Char('x') => {
                    self.unfocus_all();
                    self.oscillators.get_mut(1).unwrap().focus();
                }
                _ => (),
            };
        }
        Ok(())
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

    fn keymap(&self) -> crossterm::event::KeyCode {
        todo!()
    }
}
