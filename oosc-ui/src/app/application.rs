use std::{
    io::Stdout,
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use crate::ui::components::{root::Root, Component};

use super::{config::Config, context};
use anyhow::{Context, Result};
use cpal::traits::DeviceTrait;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, enable_raw_mode, EnterAlternateScreen},
};
use ratatui::prelude::*;

type AppTerminal = Arc<RwLock<Terminal<CrosstermBackend<Stdout>>>>;

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
        self.setup_terminal_and_hooks()?;
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

    pub fn setup_terminal_and_hooks(&mut self) -> Result<()> {
        let terminal = Self::setup_terminal().context("setup failed")?;
        Self::chain_hook();
        self.run_term(terminal.clone()).context("app loop failed")?;
        Self::restore_terminal().context("restore terminal failed")?;
        Ok(())
    }

    fn setup_terminal() -> Result<AppTerminal> {
        let mut stdout = std::io::stdout();
        enable_raw_mode().context("failed to enable raw mode")?;
        execute!(stdout, EnterAlternateScreen).context("unable to enter alternate screen")?;
        Ok(Arc::new(RwLock::new(
            Terminal::new(CrosstermBackend::new(stdout)).context("creating terminal failed")?,
        )))
    }

    fn restore_terminal() -> Result<(), anyhow::Error> {
        execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        // crossterm::terminal::show_cursor().context("unable to show cursor")
        Ok(())
    }

    fn run_term(&mut self, terminal: AppTerminal) -> Result<(), anyhow::Error> {
        let mut terminal = terminal.write().unwrap();
        let mut root = Root::new(self, &terminal.get_frame());
        loop {
            thread::sleep(Duration::from_millis(16));
            let _syn = self.ctx.synthesizer.lock().unwrap();
            if !Self::read_events(self, &mut root)? {
                break;
            }
            terminal.draw(|f| {
                let _ = root.draw(f, f.size());
            })?;
        }
        Ok(())
    }

    fn chain_hook() {
        let original_hook = std::panic::take_hook();

        std::panic::set_hook(Box::new(move |panic| {
            let restore = Self::restore_terminal();
            if restore.is_err() {
                println!(
                    "Error until restore terminal on panic: {}",
                    restore.err().unwrap()
                );
            }
            original_hook(panic);
        }));
    }

    fn read_events(&self, root: &mut Root) -> Result<bool> {
        if event::poll(Duration::from_millis(0)).context("event poll failed")? {
            let event = event::read().context("event read failed")?;
            let event_copy = event.clone();
            root.handle_events(Some(event_copy))?;
            if let Event::Key(key) = event {
                return Ok(match key.code {
                    KeyCode::Char('q') => false,
                    _ => true,
                });
            }
        }
        Ok(true)
    }
}
