pub mod oscillator;
pub mod parametr;
pub mod root;
pub mod synthesizer;
pub mod wavetable;
use anyhow::Result;
use crossterm::event::{Event, KeyEvent, MouseEvent};
use ratatui::{layout::Rect, style::Color, Frame};

pub enum EmptyAction {
    None,
}

pub trait Component {
    type Action;

    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn resize(&mut self, _rect: Rect) -> Result<()> {
        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<()> {
        match event {
            Some(Event::Key(key_event)) => self.handle_key_events(key_event),
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_events(mouse_event),
            Some(Event::Resize(x, y)) => self.resize(Rect::new(0, 0, x, y)),
            _ => Ok(()),
        }
    }

    fn handle_key_events(&mut self, _key: KeyEvent) -> Result<()> {
        Ok(())
    }

    fn handle_mouse_events(&mut self, _mouse: MouseEvent) -> Result<()> {
        Ok(())
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()>;
}

pub trait Focus {
    fn focus(&mut self);
    fn unfocus(&mut self);
    fn is_focused(&self) -> bool;
    fn color(&self) -> Color {
        if self.is_focused() {
            Color::Yellow
        } else {
            Color::Gray
        }
    }
}

pub trait FocusableComponent: Component + Focus {}
