use ratatui::Frame;

use crate::app::application::Application;

use super::{Component, FocusableComponent, synthesizer::SynthesizerComponent};

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Normal,
    Command,
}

#[derive(Clone, Copy, Debug)]
pub enum Focus {
    Default,
    Command,
}

#[derive(Clone, Copy, Debug)]
pub enum RootAction {
    SetMode(Mode),
    ChangeFocus(u32),
}

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
    type Action = RootAction;

    fn draw(
        &mut self,
        f: &mut ratatui::Frame<'_>,
        rect: ratatui::prelude::Rect,
    ) -> anyhow::Result<()> {
        self.synthesizer.draw(f, rect)
    }

    fn resize(&mut self, rect: ratatui::prelude::Rect) -> anyhow::Result<()> {
        println!("Resize root");
        self.synthesizer.resize(rect)
    }
}
