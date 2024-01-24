use std::rc::Rc;

use crossterm::event::KeyCode;
use oosc_core::{
    core::{oscillator::WavetableOscillator, synthesizer::Synthesizer},
    utils::{make_shared, Shared},
};
use ratatui::prelude::*;

use super::{
    components_container::ComponentsContainer, effect::EffectComponent, menu_bar::MenuBar,
    oscillator::OscillatorComponent, Component, Focus, FocusableComponent,
    FocusableComponentContext, NamedFocusableComponent,
};

struct SynthesizerLayout {
    rect: Rect,
    oscillators: Rc<Vec<Rect>>,
}

pub struct SynthesizerComponent {
    pub oscillators: Shared<ComponentsContainer<dyn NamedFocusableComponent>>,
    menu: MenuBar<dyn NamedFocusableComponent>,
    context: FocusableComponentContext,
    layout: Option<SynthesizerLayout>,
}

impl SynthesizerComponent {
    pub fn new(synthesizer: &Synthesizer) -> Self {
        let mut oscillators = ComponentsContainer::from(
            synthesizer
                .get_oscillators::<WavetableOscillator>()
                .enumerate()
                .map(|(i, osc)| {
                    let map = KeyCode::Char(char::from_digit(i as u32 + 1, 10).unwrap());
                    make_shared(OscillatorComponent::new(osc, map))
                        as Shared<dyn NamedFocusableComponent>
                })
                .collect::<Vec<Shared<dyn NamedFocusableComponent>>>(),
        );
        let effect = synthesizer.get_named_effects().next().unwrap();
        let effect = make_shared(EffectComponent::new(effect));
        oscillators.components.push(effect);
        oscillators.draw_only_focused(true);
        let oscillators = make_shared(oscillators);
        let menu = MenuBar::new(oscillators.clone(), "Menu");
        let context = FocusableComponentContext::new().wrapper(true);
        Self {
            oscillators,
            menu,
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
        self.menu.draw(f, _rect)?;
        let mut oscillators = self.oscillators.write().unwrap();
        oscillators.draw_in_layout(f, &layout.oscillators)?;
        Ok(())
    }

    fn resize(&mut self, rect: Rect) -> anyhow::Result<()> {
        /* let len = self.oscillators.components.len();
        let size = 100 / len;
        let oscillators = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                std::iter::repeat_with(|| Constraint::Percentage(size as u16))
                    .take(len)
                    .collect::<Vec<_>>(),
            )
            .split(rect); */
        let osc_rect = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(rect);

        let mut oscillators = self.oscillators.write().unwrap();
        let oscillators_layout = Rc::new(vec![osc_rect[1]; oscillators.components.len()]);
        oscillators.resize_in_layout(&oscillators_layout)?;
        self.menu.resize(osc_rect[0])?;
        self.layout = Some(SynthesizerLayout {
            rect,
            oscillators: oscillators_layout,
        });
        Ok(())
    }

    fn handle_key_events(&mut self, key: crossterm::event::KeyEvent) -> anyhow::Result<()> {
        let mut oscillators = self.oscillators.write().unwrap();
        oscillators.handle_key_events(key)
    }
}

impl Focus for SynthesizerComponent {
    fn focus(&mut self) {
        self.context_mut().focus()
    }

    fn unfocus(&mut self) {
        self.context_mut().unfocus()
    }

    fn is_focused(&self) -> bool {
        self.oscillators.read().unwrap().is_any_focused()
    }

    fn keymap(&self) -> Option<crossterm::event::KeyCode> {
        self.context().keymap()
    }

    fn color(&self) -> Color {
        if self.is_focused() {
            *self
                .context()
                .focused_color
                .as_ref()
                .unwrap_or(&Color::Yellow)
        } else {
            *self
                .context()
                .unfocused_color
                .as_ref()
                .unwrap_or(&Color::Gray)
        }
    }
}
