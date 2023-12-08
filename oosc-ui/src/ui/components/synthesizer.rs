use oosc_core::core::synthesizer::SyncSynthesizer;

use super::{Component, Focus, FocusableComponent};

pub enum Action {
    SelectOscillator(u8),
    SelectEffectsBlock,
}

pub struct SynthesizerComponent {
    pub synthesizer: SyncSynthesizer,
}

impl FocusableComponent for SynthesizerComponent {}

impl Component for SynthesizerComponent {
    type Action = Action;

    fn draw(
        &mut self,
        f: &mut ratatui::Frame<'_>,
        rect: ratatui::prelude::Rect,
    ) -> anyhow::Result<()> {
        todo!()
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
