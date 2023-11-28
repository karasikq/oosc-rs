use std::{io::Stdout, time::Duration};

use crate::{core::note::Note, utils::adsr_envelope::State};

use super::{config::Config, context};
use anyhow::Context;
use cpal::{traits::{DeviceTrait, StreamTrait, HostTrait}, StreamConfig};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};

pub struct Application {
    pub ctx: context::Context,
    pub config: Config,
}

impl Application {
    pub fn new() -> Result<Self, crate::error::Error> {
        let config = Config {
            channels: 2,
            sample_rate: 48000,
            delta_time: 1.0 / 48000.0,
            buffer_size: 512,
        };
        let ctx = context::Context::build_default(&config)?;
        Ok(Application { ctx, config })
    }

    pub fn run(&mut self) -> Result<(), crate::error::Error> {
        // self.detach_stream()?;
        self.detach_keyboard().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn detach_stream(&mut self) -> Result<(), crate::error::Error> {
        Ok(())
    }

    // MOVE THIS TO UI MOD
    pub fn detach_keyboard(&mut self) -> Result<(), anyhow::Error> {
        let mut terminal = self.setup_terminal().context("setup failed")?;
        self.run_term(&mut terminal).context("app loop failed")?;
        Self::restore_terminal(&mut terminal).context("restore terminal failed")?;
        Ok(())
    }

    fn setup_terminal(&mut self) -> Result<Terminal<CrosstermBackend<Stdout>>, anyhow::Error> {
        let mut stdout = std::io::stdout();
        enable_raw_mode().context("failed to enable raw mode")?;
        execute!(stdout, EnterAlternateScreen).context("unable to enter alternate screen")?;
        Terminal::new(CrosstermBackend::new(stdout)).context("creating terminal failed")
    }

    fn restore_terminal(
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), anyhow::Error> {
        disable_raw_mode().context("failed to disable raw mode")?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)
            .context("unable to switch to main screen")?;
        terminal.show_cursor().context("unable to show cursor")
    }

    fn run_term(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), anyhow::Error> {
        loop {
            terminal.draw(Self::render_app)?;
            if !Self::read_key(self)? {
                break;
            }
        }
        Ok(())
    }

    fn render_app(frame: &mut Frame) {
        let greeting = Paragraph::new("Hello World! (press 'q' to quit)");
        frame.render_widget(greeting, frame.size());
    }

    fn read_key(&mut self) -> Result<bool, anyhow::Error> {
        if event::poll(Duration::from_millis(250)).context("event poll failed")? {
            if let Event::Key(key) = event::read().context("event read failed")? {
                return Ok(match key.code {
                    KeyCode::Char('q') => false,
                    KeyCode::Char(c) => {
                        self.play_note(c as u32 - 37);
                        true
                    }
                    _ => false,
                });
            }
        }
        Ok(true)
    }

    fn play_note(&mut self, note: u32) {
        let syn = self.ctx.synthesizer.clone();
        let mut locked = syn.lock().unwrap();
        let mut note = Note::new(note, 127);
        note.hold_on = State::None;
        println!("Play note: {}", note.note);
        locked.note_on(note).unwrap();
    }
}
