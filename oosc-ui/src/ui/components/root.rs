use crate::app::application::Application;

use super::{Component, FocusableComponent};

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

pub struct Root<'a> {
    pub app: &'a Application,
}

impl<'a> Root<'a> {
    pub fn new(app: &'a Application) {}
}
