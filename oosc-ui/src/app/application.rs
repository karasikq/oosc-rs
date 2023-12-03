use std::{io::Stdout, time::Duration};

use oosc_core::{core::note::Note, utils::adsr_envelope::State};

use crate::ui::renderer::Renderer;

use super::{config::Config, context};
use anyhow::{Context, Result};
use cpal::traits::DeviceTrait;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

pub struct Application {
    pub ctx: context::Context,
    pub config: Config,
}

impl Application {
    pub fn new() -> Result<Self> {
        let config = Config {
            channels: 2,
            sample_rate: 48000,
            delta_time: 1.0 / 48000.0,
            buffer_size: 512,
        };
        let ctx = context::Context::build_default(&config)?;
        Ok(Application { ctx, config })
    }

    pub fn run(&mut self) -> Result<()> {
        self.detach_keyboard()?;
        Ok(())
    }

    pub fn detach_stream(&mut self) -> Result<cpal::Stream> {
        let (_, device, config) = context::Context::get_default_device(&self.config)?;
        let err_fn = |err| println!("an error occurred on stream: {}", err);
        let callbacks = self.ctx.callbacks.get_callbacks();
        let mut total_playback_seconds = 0.;
        let delta = self.config.buffer_size as f32 / self.config.sample_rate as f32;
        Ok(device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                callbacks
                    .iter()
                    .try_for_each(|callback| -> Result<()> {
                        let mut callback = callback.lock().unwrap();
                        Ok(callback.process_stream(data, total_playback_seconds)?)
                    })
                    .unwrap();
                total_playback_seconds += delta;
            },
            err_fn,
            None,
        )?)
    }

    // MOVE THIS TO UI MOD
    pub fn detach_keyboard(&mut self) -> Result<()> {
        let mut terminal = self.setup_terminal().context("setup failed")?;
        self.run_term(&mut terminal).context("app loop failed")?;
        Self::restore_terminal(&mut terminal).context("restore terminal failed")?;
        Ok(())
    }

    fn setup_terminal(&mut self) -> Result<Terminal<CrosstermBackend<Stdout>>> {
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
            let render = Renderer::new(self);
            terminal.draw(|f| {
                Self::render_app(f, render);
            })?;
            if !Self::read_key(self)? {
                break;
            }
        }
        Ok(())
    }

    fn render_app(frame: &mut Frame, renderer: Renderer) {
        frame.render_widget(renderer, frame.size());
    }

    fn read_key(&mut self) -> Result<bool> {
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
