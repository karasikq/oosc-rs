use std::rc::Rc;

use crossterm::event::KeyCode;
use oosc_core::core::{oscillator::WavetableOscillator, synthesizer::Synthesizer};
use ratatui::prelude::*;

use super::{
    components_container::ComponentsContainer, oscillator::OscillatorComponent, Component,
    FocusableComponent, FocusableComponentContext,
};

struct SynthesizerLayout {
    rect: Rect,
    oscillators: Rc<[Rect]>,
}

pub struct SynthesizerComponent {
    pub oscillators: ComponentsContainer<OscillatorComponent>,
    context: FocusableComponentContext,
    layout: Option<SynthesizerLayout>,
}

impl SynthesizerComponent {
    pub fn new(synthesizer: &mut Synthesizer) -> Self {
        let oscillators = ComponentsContainer::from(
            synthesizer
                .get_oscillators::<WavetableOscillator>()
                .enumerate()
                .map(|(i, osc)| {
                    let map = KeyCode::Char(char::from_digit(i as u32 + 1, 10).unwrap());
                    OscillatorComponent::new(osc, map)
                })
                .collect::<Vec<OscillatorComponent>>(),
        );
        let context = FocusableComponentContext::new();
        Self {
            oscillators,
            context,
            layout: None,
        }
    }
}

impl FocusableComponent for SynthesizerComponent {
    fn context(&self) -> &FocusableComponentContext {
        &self.context
    }

    fn context_mut(&mut self) -> &mut FocusableComponentContext {
        &mut self.context
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Component for SynthesizerComponent {
    fn draw(
        &mut self,
        f: &mut ratatui::Frame<'_>,
        _rect: ratatui::prelude::Rect,
    ) -> anyhow::Result<()> {
        if self.layout.is_none() {
            return Err(oosc_core::error::Error::from("Create layout before draw"))?;
        }
        let layout = self.layout.as_ref().unwrap();
        let _rect = layout.rect;
        self.oscillators.draw_in_layout(f, &layout.oscillators)?;
        Ok(())
    }

    fn resize(&mut self, rect: Rect) -> anyhow::Result<()> {
        let len = self.oscillators.components.len();
        let size = 100 / len;
        let oscillators = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                std::iter::repeat_with(|| Constraint::Percentage(size as u16))
                    .take(len)
                    .collect::<Vec<_>>(),
            )
            .split(rect);
        self.oscillators.resize_in_layout(&oscillators)?;
        self.layout = Some(SynthesizerLayout { rect, oscillators });
        Ok(())
    }

    fn handle_key_events(&mut self, key: crossterm::event::KeyEvent) -> anyhow::Result<()> {
        self.oscillators.handle_key_events(key)
    }
}
