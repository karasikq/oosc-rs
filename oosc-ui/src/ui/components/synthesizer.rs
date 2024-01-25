use crossterm::event::KeyCode;
use oosc_core::{
    core::{oscillator::WavetableOscillator, synthesizer::Synthesizer},
    utils::{make_shared, Shared},
};
use ratatui::prelude::*;

use super::{
    components_container::ComponentsContainer, effects_container::EffectsContainer,
    menu_bar::MenuBar, oscillator::OscillatorComponent, Component, Focus, FocusableComponent,
    FocusableComponentContext, NamedFocusableComponent,
};

struct SynthesizerLayout {
    rect: Rect,
}

pub struct SynthesizerComponent {
    pub components: Shared<ComponentsContainer<dyn NamedFocusableComponent>>,
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
        let effects = synthesizer.get_named_effects();
        let effects = make_shared(EffectsContainer::new(effects));
        oscillators.components.push(effects);
        oscillators.draw_only_focused(true);
        let oscillators = make_shared(oscillators);
        let menu = MenuBar::new(oscillators.clone(), "Menu");
        let context = FocusableComponentContext::new().wrapper(true);
        Self {
            components: oscillators,
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
        rect: ratatui::prelude::Rect,
    ) -> anyhow::Result<()> {
        if self.layout.is_none() {
            return Err(oosc_core::error::Error::from("Create layout before draw"))?;
        }
        let _rect = self.layout.as_ref().unwrap().rect;
        self.menu.draw(f, rect)?;
        let mut oscillators = self.components.write().unwrap();
        oscillators.draw(f, rect)?;
        Ok(())
    }

    fn resize(&mut self, rect: Rect) -> anyhow::Result<()> {
        let osc_rect = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(rect);

        let mut oscillators = self.components.write().unwrap();
        oscillators.resize(osc_rect[1])?;
        self.menu.resize(osc_rect[0])?;
        self.layout = Some(SynthesizerLayout { rect });
        Ok(())
    }

    fn handle_key_events(&mut self, key: crossterm::event::KeyEvent) -> anyhow::Result<()> {
        let mut oscillators = self.components.write().unwrap();
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
        self.components.read().unwrap().is_any_focused()
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
