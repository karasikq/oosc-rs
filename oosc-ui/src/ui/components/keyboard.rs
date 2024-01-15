use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use midir::{MidiOutput, MidiOutputConnection};
use ratatui::{
    prelude::{Alignment, Margin, Rect},
    style::Style,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::ui::utils::keycode_to_string;

use super::{AutoFocus, Component, Focus, FocusableComponent, FocusableComponentContext};

struct KeyboardLayout {
    pub rect: Rect,
    pub inner: Rect,
}

pub struct KeyboardComponent {
    output: MidiOutputConnection,
    ctx: FocusableComponentContext,
    layout: Option<KeyboardLayout>,
    last_note: Option<i32>,
}

impl KeyboardComponent {
    pub fn new(keymap: KeyCode) -> Result<Self> {
        let ctx = FocusableComponentContext::new().keymap(keymap);
        let output = MidiOutput::new("oosc")?;
        let out_ports = output.ports();
        let output = output.connect(&out_ports[0], "oosc-output").unwrap();
        Ok(Self {
            output,
            ctx,
            layout: None,
            last_note: None,
        })
    }
}

impl Component for KeyboardComponent {
    fn draw(
        &mut self,
        f: &mut ratatui::Frame<'_>,
        _rect: ratatui::prelude::Rect,
    ) -> anyhow::Result<()> {
        let layout = self.layout.as_ref().unwrap();
        if let Some(last_note) = self.last_note {
            let p = Paragraph::new(format!("{}", last_note))
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Center);
            f.render_widget(p, layout.inner);
        }
        let b = Block::default()
            .borders(Borders::ALL)
            .title(format!(
                "midi[{}]",
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
        self.layout = Some(KeyboardLayout { rect, inner });
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        match key.code {
            KeyCode::Esc => self.unfocus(),
            KeyCode::Char(c) => {
                if key.kind == KeyEventKind::Press {
                    let _ = self.output.send(&[0x90, 60, 0x64]);
                    self.last_note = Some(60);
                }
                if key.kind == KeyEventKind::Release {
                    let _ = self.output.send(&[0x80, 60, 0x64]);
                    self.last_note = None;
                }
            }
            _ => (),
        };
        Ok(())
    }
}

impl AutoFocus for KeyboardComponent {}

impl FocusableComponent for KeyboardComponent {
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
