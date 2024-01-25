use anyhow::anyhow;
use crossterm::event::KeyCode;
use oosc_core::{
    effects::Effect,
    utils::{make_shared, Shared},
};
use ratatui::prelude::*;

use crate::ui::utils::keycode_to_string_prefixed;

use super::{
    components_container::ComponentsContainer, effect::EffectComponent, AutoFocus, Component,
    Focus, FocusableComponent, FocusableComponentContext, Named, NamedFocusableComponent,
};

struct EffectsLayout {
    rect: Rect,
}

pub struct EffectsContainer {
    pub components: Shared<ComponentsContainer<dyn NamedFocusableComponent>>,
    context: FocusableComponentContext,
    layout: Option<EffectsLayout>,
}

impl EffectsContainer {
    pub fn new(effects: impl Iterator<Item = Shared<dyn Effect>>) -> Self {
        let components = ComponentsContainer::from(
            effects
                .enumerate()
                .map(|(i, effect)| {
                    let map = KeyCode::Char(char::from_digit(i as u32 + 1, 10).unwrap());
                    make_shared(EffectComponent::new(effect, map))
                        as Shared<dyn NamedFocusableComponent>
                })
                .collect::<Vec<Shared<dyn NamedFocusableComponent>>>(),
        );
        let components = make_shared(components);
        let context = FocusableComponentContext::new().keymap(KeyCode::Char('e'));
        Self {
            components,
            context,
            layout: None,
        }
    }
}

impl FocusableComponent for EffectsContainer {
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

impl Component for EffectsContainer {
    fn draw(
        &mut self,
        f: &mut ratatui::Frame<'_>,
        rect: ratatui::prelude::Rect,
    ) -> anyhow::Result<()> {
        if self.layout.is_none() {
            return Err(
                anyhow!("Cannot draw effects container").context("Create layout before draw")
            );
        }
        let _rect = self.layout.as_ref().unwrap().rect;
        let mut components = self.components.write().unwrap();
        components.draw(f, rect)?;
        Ok(())
    }

    fn resize(&mut self, rect: Rect) -> anyhow::Result<()> {
        let inner = {
            let len = self.components.read().unwrap().components.len();
            let size = 100 / len;
            Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Percentage(size as u16); len])
                .split(rect)
        };
        let mut components = self.components.write().unwrap();
        components.resize_in_layout(&inner)?;
        self.layout = Some(EffectsLayout { rect });
        Ok(())
    }

    fn handle_key_events(&mut self, key: crossterm::event::KeyEvent) -> anyhow::Result<()> {
        if !self.components.read().unwrap().is_any_focused() && key.code == KeyCode::Esc {
            self.unfocus()
        }
        let mut components = self.components.write().unwrap();
        components.handle_key_events(key)
    }
}

impl Named for EffectsContainer {
    fn name(&self) -> Vec<Span<'static>> {
        vec![
            Span::styled("Effects", Style::default().fg(self.color())),
            Span::styled(
                keycode_to_string_prefixed(self.keymap(), "[", "]"),
                Style::default().fg(Color::Red),
            ),
        ]
    }
}

impl AutoFocus for EffectsContainer {}
