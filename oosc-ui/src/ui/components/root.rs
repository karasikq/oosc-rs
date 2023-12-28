use ratatui::Frame;

use crate::app::application::Application;

use super::{Component, synthesizer::SynthesizerComponent};

pub struct Root {
    pub synthesizer: SynthesizerComponent,
}

impl Root {
    pub fn new(app: &mut Application, frame: &Frame<'_>) -> Self {
        let mut synthesizer = app.ctx.synthesizer.lock().unwrap();
        let synthesizer = SynthesizerComponent::new(&mut synthesizer, frame.size());
        Self {
            synthesizer, 
        }
    }
}

impl Component for Root {
    fn draw(
        &mut self,
        f: &mut ratatui::Frame<'_>,
        rect: ratatui::prelude::Rect,
    ) -> anyhow::Result<()> {
        self.synthesizer.draw(f, rect)
    }

    fn resize(&mut self, rect: ratatui::prelude::Rect) -> anyhow::Result<()> {
        self.synthesizer.resize(rect)
    }

    fn handle_key_events(&mut self, key: crossterm::event::KeyEvent) -> anyhow::Result<()> {
        self.synthesizer.handle_key_events(key)
    }
}
