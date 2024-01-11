use crossterm::event::KeyCode;
use oosc_core::utils::{make_shared, Shared};
use ratatui::prelude::*;

use crate::app::context::Context;

use super::{
    components_container::ComponentsContainer, record::RecordComponent,
    synthesizer::SynthesizerComponent, Component, FocusableComponent,
};

pub struct Root {
    pub synthesizer: Shared<SynthesizerComponent>,
    pub recorder: Shared<RecordComponent>,
    components: ComponentsContainer<dyn FocusableComponent>,
}

impl Root {
    pub fn new(ctx: &mut Context) -> Self {
        let synthesizer = ctx.synthesizer.lock().unwrap();
        let synthesizer = make_shared(SynthesizerComponent::new(&synthesizer));
        let recorder = ctx.render_control.clone();
        let recorder = make_shared(RecordComponent::new(recorder, KeyCode::Char('r')));
        let mut components = ComponentsContainer::new();
        components
            .container()
            .push(synthesizer.clone() as Shared<dyn FocusableComponent>);
        components
            .container()
            .push(recorder.clone() as Shared<dyn FocusableComponent>);
        Self {
            synthesizer,
            recorder,
            components,
        }
    }
}

impl Component for Root {
    fn draw(
        &mut self,
        f: &mut ratatui::Frame<'_>,
        rect: ratatui::prelude::Rect,
    ) -> anyhow::Result<()> {
        self.components.draw(f, rect)
    }

    fn resize(&mut self, rect: ratatui::prelude::Rect) -> anyhow::Result<()> {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .margin(1)
            .split(rect);
        self.components.resize_in_layout(&layout)
    }

    fn handle_key_events(&mut self, key: crossterm::event::KeyEvent) -> anyhow::Result<()> {
        self.components.handle_key_events(key)
    }
}
