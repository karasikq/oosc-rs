pub mod bezier;
pub mod components_container;
pub mod envelope;
pub mod oscillator;
pub mod parameter;
pub mod root;
pub mod synthesizer;
pub mod wavetable;
use std::any::Any;

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, MouseEvent};
use oosc_core::utils::Shared;
use ratatui::{layout::Rect, style::Color, Frame};

pub trait Component {
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
    fn color(&self) -> Color;
    fn keymap(&self) -> Option<crossterm::event::KeyCode>;
}

pub type SharedComponent = Shared<dyn FocusableComponent>;
pub type FocusCallback<T> = Option<Box<dyn FnMut(T)>>;

pub struct FocusableComponentContext {
    pub keymap: Option<KeyCode>,
    pub focused_color: Option<Color>,
    pub unfocused_color: Option<Color>,
    pub focused: bool,
}

impl FocusableComponentContext {
    pub fn new() -> FocusableComponentContext {
        FocusableComponentContext {
            keymap: None,
            focused_color: None,
            unfocused_color: None,
            focused: false,
        }
    }

    pub fn keymap(self, keymap: KeyCode) -> FocusableComponentContext {
        FocusableComponentContext {
            keymap: Some(keymap),
            ..self
        }
    }

    pub fn focused_color(self, color: Color) -> FocusableComponentContext {
        FocusableComponentContext {
            focused_color: Some(color),
            ..self
        }
    }

    pub fn unfocused_color(self, color: Color) -> FocusableComponentContext {
        FocusableComponentContext {
            unfocused_color: Some(color),
            ..self
        }
    }

    pub fn focused(self, focused: bool) -> FocusableComponentContext {
        FocusableComponentContext { focused, ..self }
    }
}

impl Default for FocusableComponentContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Focus for FocusableComponentContext {
    fn focus(&mut self) {
        self.focused = true
    }

    fn unfocus(&mut self) {
        self.focused = false
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn keymap(&self) -> Option<crossterm::event::KeyCode> {
        self.keymap
    }

    fn color(&self) -> Color {
        if self.is_focused() {
            Color::Yellow
        } else {
            Color::Gray
        }
    }
}

pub trait FocusableComponent: Component + Focus {
    fn context(&self) -> &FocusableComponentContext;
    fn context_mut(&mut self) -> &mut FocusableComponentContext;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: FocusableComponent> Focus for T {
    fn focus(&mut self) {
        self.context_mut().focus()
    }

    fn unfocus(&mut self) {
        self.context_mut().unfocus()
    }

    fn is_focused(&self) -> bool {
        self.context().is_focused()
    }

    fn keymap(&self) -> Option<crossterm::event::KeyCode> {
        self.context().keymap()
    }

    fn color(&self) -> Color {
        if self.is_focused() {
            *self
                .context()
                .focused_color
                .as_ref()
                .unwrap_or(&Color::Yellow)
        } else {
            *self
                .context()
                .unfocused_color
                .as_ref()
                .unwrap_or(&Color::Gray)
        }
    }
}
