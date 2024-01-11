use crossterm::event::{KeyCode, KeyEvent};
use oosc_core::{
    callbacks::stream_renderer::{RenderState, StreamRenderer, StreamWavRenderer},
    utils::SharedMutex,
};
use ratatui::{
    prelude::{Alignment, Margin, Rect},
    style::Style,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::ui::utils::keycode_to_string;

use super::{AutoFocus, Component, Focus, FocusableComponent, FocusableComponentContext};

struct RecordLayout {
    pub rect: Rect,
    pub inner: Rect,
}

pub struct RecordComponent {
    control: SharedMutex<StreamWavRenderer>,
    ctx: FocusableComponentContext,
    layout: Option<RecordLayout>,
}

impl RecordComponent {
    pub fn new(control: SharedMutex<StreamWavRenderer>, keymap: KeyCode) -> Self {
        let ctx = FocusableComponentContext::new().keymap(keymap);
        Self {
            control,
            ctx,
            layout: None,
        }
    }
}

impl Component for RecordComponent {
    fn draw(
        &mut self,
        f: &mut ratatui::Frame<'_>,
        _rect: ratatui::prelude::Rect,
    ) -> anyhow::Result<()> {
        let layout = self.layout.as_ref().unwrap();
        let p = {
            let control = self.control.lock().unwrap();
            match control.get_state() {
                RenderState::None => Paragraph::new("Record [r] Stop [s]")
                    .wrap(Wrap { trim: true })
                    .alignment(Alignment::Center),
                RenderState::Rendering => {
                    Paragraph::new(format!("Recording {:.2}s Stop [s]", control.time()))
                        .wrap(Wrap { trim: true })
                        .alignment(Alignment::Center)
                }
            }
        };
        f.render_widget(p, layout.inner);
        let b = Block::default()
            .borders(Borders::ALL)
            .title(format!(
                "recorder[{}]",
                keycode_to_string(self.keymap().unwrap_or(KeyCode::Null))
            ))
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(self.color()));
        f.render_widget(b, layout.rect);
        Ok(())
    }

    fn resize(&mut self, rect: Rect) -> anyhow::Result<()> {
        let inner = rect.inner(&Margin {
            horizontal: 1,
            vertical: 1,
        });
        self.layout = Some(RecordLayout { rect, inner });
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        match key.code {
            KeyCode::Esc => self.unfocus(),
            KeyCode::Char(c) => {
                match c {
                    'r' => {
                        let mut control = self.control.lock().unwrap();
                        control.to_file("record.wav")?;
                        let _ = control.start();
                    }
                    's' => {
                        let mut control = self.control.lock().unwrap();
                        let _ = control.stop();
                    }
                    _ => (),
                };
            }
            _ => (),
        };
        Ok(())
    }
}

impl AutoFocus for RecordComponent {}

impl FocusableComponent for RecordComponent {
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
