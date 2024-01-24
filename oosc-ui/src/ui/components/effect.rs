use crossterm::event::KeyCode;
use oosc_core::{
    effects::Effect,
    utils::{make_shared, Shared},
};
use ratatui::{
    prelude::{Alignment, Direction, Margin, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, BorderType, Borders},
};

use crate::ui::{components::Focus, utils::keycode_to_string_prefixed};

use super::{
    components_container::ComponentsContainer, parameter::ParameterComponentF32, AutoFocus,
    Component, FocusableComponent, FocusableComponentContext, Named,
};

struct EffectLayout {
    pub rect: Rect,
    pub inner: Rect,
}

pub struct EffectComponent {
    effect: Shared<dyn Effect>,
    parameters: ComponentsContainer<dyn FocusableComponent>,
    ctx: FocusableComponentContext,
    layout: Option<EffectLayout>,
}

impl EffectComponent {
    pub fn new(effect: Shared<dyn Effect>) -> Self {
        let parameters = {
            let mut effect_guard = effect.write().unwrap();
            let parameters_container = effect_guard.parameters().unwrap().parameters_f32().unwrap();
            let parameters = parameters_container
                .iter()
                .map(|p| {
                    make_shared(ParameterComponentF32::from_named(
                        p,
                        Direction::Vertical,
                        20,
                        oosc_core::utils::interpolation::InterpolateMethod::Linear,
                        KeyCode::Char('1'),
                    )) as Shared<dyn FocusableComponent>
                })
                .collect::<Vec<Shared<dyn FocusableComponent>>>();
            ComponentsContainer::from(parameters)
        };
        Self {
            effect,
            parameters,
            ctx: FocusableComponentContext::new().keymap(KeyCode::Char('e')),
            layout: None,
        }
    }
}

impl Component for EffectComponent {
    fn draw(
        &mut self,
        f: &mut ratatui::Frame<'_>,
        _rect: ratatui::prelude::Rect,
    ) -> anyhow::Result<()> {
        let layout = self.layout.as_ref().unwrap();
        let b = Block::default()
            .borders(Borders::ALL)
            .title(self.effect.read().unwrap().name())
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center);
        f.render_widget(b, layout.rect);
        self.parameters.draw_in_layout(f, &[layout.inner])?;
        Ok(())
    }

    fn resize(&mut self, rect: ratatui::prelude::Rect) -> anyhow::Result<()> {
        let inner = rect.inner(&Margin::new(1, 1));
        self.parameters.resize_in_layout(&[inner, inner])?;
        self.layout = Some(EffectLayout { rect, inner });
        Ok(())
    }

    fn handle_key_events(&mut self, key: crossterm::event::KeyEvent) -> anyhow::Result<()> {
        if !self.parameters.is_any_focused() && key.code == KeyCode::Esc {
            self.unfocus()
        }
        self.parameters.handle_key_events(key)
    }
}

impl AutoFocus for EffectComponent {}

impl FocusableComponent for EffectComponent {
    fn context(&self) -> &FocusableComponentContext {
        &self.ctx
    }

    fn context_mut(&mut self) -> &mut FocusableComponentContext {
        &mut self.ctx
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Named for EffectComponent {
    fn name(&self) -> Vec<Span<'static>> {
        vec![
            Span::styled(
                self.effect.read().unwrap().name(),
                Style::default().fg(self.color()),
            ),
            Span::styled(
                keycode_to_string_prefixed(self.keymap(), "[", "]"),
                Style::default().fg(Color::Red),
            ),
        ]
    }
}
