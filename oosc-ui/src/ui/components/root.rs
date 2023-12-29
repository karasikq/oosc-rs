use oosc_core::{core::synthesizer::Synthesizer, utils::SharedMutex};

use super::{synthesizer::SynthesizerComponent, Component};

pub struct Root {
    pub synthesizer: SynthesizerComponent,
}

impl Root {
    pub fn new(synthesizer: SharedMutex<Synthesizer>) -> Self {
        let mut synthesizer = synthesizer.lock().unwrap();
        let synthesizer = SynthesizerComponent::new(&mut synthesizer);
        Self { synthesizer }
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
